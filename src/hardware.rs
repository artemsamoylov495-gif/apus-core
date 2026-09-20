use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind, Disks};

#[derive(Debug)]
pub struct HardwareScore {
    pub total_ram_gb: u64,
    pub logical_cores: usize,
    pub is_ssd: bool,
    pub can_participate: bool,
}

pub fn evaluate_node() -> HardwareScore {
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    sys.refresh_memory();
    sys.refresh_cpu_usage();

    let total_ram_gb = sys.total_memory() / 1024 / 1024 / 1024;
    let logical_cores = sys.cpus().len();

    // Проверка наличия SSD
    let disks = Disks::new_with_refreshed_list();
    let is_ssd = disks.iter().any(|disk| {
        let kind = disk.kind().to_string();
        kind.contains("SSD") || kind.contains("NVMe") || disk.file_system().to_string_lossy().contains("ext4")
    });

    // Пороги APUS: ОЗУ >= 8 ГБ, Потоков >= 6, Накопитель = SSD
    let can_participate = total_ram_gb >= 8 && logical_cores >= 6 && is_ssd;

    HardwareScore {
        total_ram_gb,
        logical_cores,
        is_ssd,
        can_participate,
    }
}
