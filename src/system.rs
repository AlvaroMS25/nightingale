use parking_lot::RwLock;
use sysinfo::{Pid, ProcessRefreshKind, System as SysInfo};
use crate::api::model::info::{CoreInfo, CpuInfo, MemoryInfo, SystemInfo};

pub struct System {
    inner: RwLock<SystemInner>
}

struct SystemInner {
    inner: SysInfo,
    pid: [Pid; 1]
}

impl SystemInner {
    fn update(&mut self) {
        self.inner.refresh_pids_specifics(&self.pid, ProcessRefreshKind::new()
            .with_cpu()
            .with_memory()
        );
        self.inner.refresh_cpu();
        self.inner.refresh_memory();
    }

    fn get(&self) -> SystemInfo {
        let process = self.inner.process(self.pid[0]).unwrap();

        let cpu = CpuInfo {
            total_usage: self.inner.global_cpu_info().cpu_usage(),
            process_usage: process.cpu_usage(),
            cores: self.inner.cpus().iter()
                .map(|cpu| {
                    CoreInfo {
                        total_usage: cpu.cpu_usage(),
                        frequency: cpu.frequency()
                    }
                })
                .collect()
        };

        let mem = MemoryInfo {
            memory: process.memory(),
            virtual_memory: process.virtual_memory()
        };

        SystemInfo {
            cpu,
            memory: mem
        }
    }
}

impl System {
    pub fn new(pid: Pid) -> Self {
        Self {
            inner: RwLock::new(SystemInner {
                inner: SysInfo::new_all(),
                pid: [pid]
            }),
        }
    }

    pub fn update_get(&self) -> SystemInfo {
        let mut write = self.inner.write();
        write.update();
        write.get()
    }

    pub fn update(&self) {
        self.inner.write().update();
    }

    pub fn get(&self) -> SystemInfo {
        self.inner.read().get()
    }
}
