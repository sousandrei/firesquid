mod child;
mod drive;
mod http;
mod socket;

use tokio::task;
use tracing::{error, info};

use crate::error::RuntimeError;
use crate::State;
use crate::Vm;

pub async fn spawn(name: &str, state: State) -> Result<(), RuntimeError> {
    let name = String::from(name);

    {
        let vms = state.vms.lock().unwrap();
        if let Some(_) = vms.iter().position(|vm| vm.name == name) {
            return Err(RuntimeError::new(&format!(
                "Vm name already used [{}]",
                name
            )));
        }
    }

    if let Err(_) = drive::create_drive(&name, &state.tmp_dir, &state.assets_dir) {
        drive::delete_drive(&name, &state.tmp_dir)?;
        socket::delete_socket(&name, &state.tmp_dir).unwrap();

        return Err(RuntimeError::new("Error creating drive"));
    };

    task::spawn(async move {
        let child = child::spawn_process(&name, state.clone()).await.unwrap();
        {
            let mut vms = state.vms.lock().unwrap();

            vms.push(Vm {
                name: name.clone(),
                pid: child.id(),
            });
        }

        if let Err(_) = child.await {
            error!(
                "Failed to start machine, proceeding to teardown [{}]",
                &name
            );
        };

        drive::delete_drive(&name, &state.tmp_dir).unwrap();
        socket::delete_socket(&name, &state.tmp_dir).unwrap();

        let mut vms = state.vms.lock().unwrap();
        if let Some(index) = vms.iter().position(|vm| vm.name == name) {
            vms.remove(index);
        }

        info!("Terminated [{}]", name);
    });

    Ok(())
}

pub async fn terminate(name: &str) -> Result<(), RuntimeError> {
    child::stop_machine(name).await?;
    Ok(())
}
