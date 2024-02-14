use serde::Serialize;

/// Information object returned from the `info` route.
#[derive(Debug, Serialize)]
pub struct Info {
    /// System information.
    pub system: SystemInfo,
    /// Playback information.
    pub playback: PlaybackInfo
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    /// Cpu information.
    pub cpu: CpuInfo,
    /// Memory information.
    pub memory: MemoryInfo
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    /// Total cpu usage.
    pub total_usage: f32,
    /// Cpu usage from `nightingale`
    pub process_usage: f32,
    /// Per-core information.
    pub cores: Vec<CoreInfo>
}

#[derive(Debug, Serialize)]
pub struct CoreInfo {
    /// Total usage of the core.
    pub total_usage: f32,
    /// Core frequency in MHz.
    pub frequency: u64,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    /// Memory usage (RSS) in bytes, see [`Process#memory`](sysinfo::Process::memory).
    pub memory: u64,
    /// Virtual memory in bytes, see [`Process#virtual_memory`](sysinfo::Process::virtual_memory)
    pub virtual_memory: u64
}

#[derive(Debug, Serialize)]
pub struct PlaybackInfo {
    /// Number of existing players.
    pub players: u64,
    /// Number of players currently playing.
    pub playing: u64
}
