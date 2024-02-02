use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Info {
    pub system: SystemInfo,
    pub playback: PlaybackInfo
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub total_usage: f32,
    pub process_usage: f32,
    pub cores: Vec<CoreInfo>
}

#[derive(Debug, Serialize)]
pub struct CoreInfo {
    pub total_usage: f32,
    pub frequency: u64,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub memory: u64,
    pub virtual_memory: u64
}

#[derive(Debug, Serialize)]
pub struct PlaybackInfo {
    pub players: u64,
    pub playing: u64
}
