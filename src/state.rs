use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vm {
    pub name: String,
    pub pid: u32,
}

pub type State = Arc<RwLock<Vec<Vm>>>;

pub type StatePtr = Arc<State>;

pub async fn get_vms(state_ptr: StatePtr) -> Vec<Vm> {
    let vms = state_ptr.read().await;

    let mut new_vms: Vec<Vm> = Vec::new();
    for (_, item) in vms.iter().enumerate() {
        new_vms.push(item.clone());
    }

    return new_vms;
}

pub async fn add_vm(state_ptr: StatePtr, name: &str, pid: u32) {
    let mut vms = state_ptr.write().await;

    vms.push(Vm {
        name: String::from(name),
        pid,
    });
}

pub async fn remove_vm(state_ptr: StatePtr, name: &str) {
    let mut vms = state_ptr.write().await;

    if let Some(index) = vms.iter().position(|vm| vm.name == name) {
        vms.remove(index);
    }
}

pub async fn get_vm_pid(state_ptr: StatePtr, name: &str) -> Option<u32> {
    let vms = state_ptr.read().await;

    if let Some(index) = vms.iter().position(|vm| vm.name == name) {
        return Option::Some(vms[index].pid);
    } else {
        return Option::None;
    }
}
