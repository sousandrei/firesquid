use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    state::{Vm, VmStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VmMetrics {
    pub cpu_usage_percent: f64,
    pub memory_bytes: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub network_received_bytes: u64,
    pub network_transmitted_bytes: u64,
}

impl VmMetrics {
    fn zero() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_bytes: 0,
            read_bytes: 0,
            write_bytes: 0,
            network_received_bytes: 0,
            network_transmitted_bytes: 0,
        }
    }
}

pub async fn collect_metrics(vm: &Vm) -> Result<VmMetrics, Error> {
    if vm.status != VmStatus::Running {
        return Ok(VmMetrics::zero());
    }
    let Some(pid) = vm.pid else {
        return Ok(VmMetrics::zero());
    };
    tokio::task::spawn_blocking(move || collect_process_metrics(pid))
        .await
        .map_err(|error| Error::Runtime(format!("metrics task failed: {error}")))?
}

fn collect_process_metrics(pid: u32) -> Result<VmMetrics, Error> {
    let first = ProcessMetrics::read(pid)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let second = ProcessMetrics::read(pid)?;
    let cpu_delta = second.cpu_ticks.saturating_sub(first.cpu_ticks);
    let cpu_usage_percent = cpu_delta as f64 * 10.0;

    Ok(VmMetrics {
        cpu_usage_percent,
        memory_bytes: second.memory_bytes,
        read_bytes: second.read_bytes,
        write_bytes: second.write_bytes,
        network_received_bytes: 0,
        network_transmitted_bytes: 0,
    })
}

struct ProcessMetrics {
    cpu_ticks: u64,
    memory_bytes: u64,
    read_bytes: u64,
    write_bytes: u64,
}

impl ProcessMetrics {
    fn read(pid: u32) -> Result<Self, Error> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let fields = stat
            .rsplit_once(')')
            .map(|(_, fields)| fields.split_whitespace().collect::<Vec<_>>())
            .ok_or_else(|| Error::Runtime("invalid process stat data".to_owned()))?;
        let utime: u64 = fields
            .get(11)
            .ok_or_else(|| Error::Runtime("invalid process CPU data".to_owned()))?
            .parse()
            .map_err(|_| Error::Runtime("invalid process CPU data".to_owned()))?;
        let stime: u64 = fields
            .get(12)
            .ok_or_else(|| Error::Runtime("invalid process CPU data".to_owned()))?
            .parse()
            .map_err(|_| Error::Runtime("invalid process CPU data".to_owned()))?;
        let status = std::fs::read_to_string(format!("/proc/{pid}/status"))?;
        let memory_kib = status
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:")?.split_whitespace().next())
            .ok_or_else(|| Error::Runtime("invalid process memory data".to_owned()))?
            .parse::<u64>()
            .map_err(|_| Error::Runtime("invalid process memory data".to_owned()))?;
        let io = std::fs::read_to_string(format!("/proc/{pid}/io"))?;
        let read_bytes = io_value(&io, "read_bytes")?;
        let write_bytes = io_value(&io, "write_bytes")?;

        Ok(Self {
            cpu_ticks: utime + stime,
            memory_bytes: memory_kib * 1024,
            read_bytes,
            write_bytes,
        })
    }
}

fn io_value(contents: &str, key: &str) -> Result<u64, Error> {
    contents
        .lines()
        .find_map(|line| {
            line.strip_prefix(&format!("{key}:"))?
                .split_whitespace()
                .next()
        })
        .ok_or_else(|| Error::Runtime(format!("invalid process I/O data: {key}")))?
        .parse()
        .map_err(|_| Error::Runtime(format!("invalid process I/O data: {key}")))
}

#[cfg(test)]
mod tests {
    use super::io_value;

    #[test]
    fn reads_io_counter() {
        assert_eq!(io_value("read_bytes: 42\n", "read_bytes").unwrap(), 42);
    }
}
