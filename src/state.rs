use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
pub struct Vm {
    pub name: String,
    pub pid: u32,
}

pub type StatePtr = Arc<Mutex<State>>;

pub struct State {
    pub vms: Vec<Vm>,
    pub tmp_dir: String,
    pub log_dir: String,
    pub assets_dir: String,
    pub drive_name: String,
    pub kernel_name: String,
}
