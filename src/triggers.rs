use sysinfo::{System, RefreshKind, CpuRefreshKind};

#[derive(Debug, PartialEq)]
pub enum WorkMode {
    Inactive,      // 50% раздачи (10 мин простоя)
    PassiveStream, // 30% лимит (фильм/сериал)
    ActiveStop,    // 0% (игрульке / работа)
}

pub struct TriggerMonitor {
    sys: System,
}

impl TriggerMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
            ),
        }
    }

    pub fn check_current_mode(&mut self, is_user_idle_10min: bool, is_fullscreen_media: bool) -> WorkMode {
        self.sys.refresh_cpu_usage();

        let global_cpu_usage = self.sys.global_cpu_info().cpu_usage();

        // 1. Если запущен тяжелый софт или игра (CPU > 50%) — моментальное глушение раздачи
        if global_cpu_usage > 50.0 {
            return WorkMode::ActiveStop;
        }

        // 2. Если юзер афк 10 минут — включение раздачи в полсилы
        if is_user_idle_10min {
            return WorkMode::Inactive;
        }

        // 3. Пассивный просмотр: видео в полноэкране + CPU < 15%
        if is_fullscreen_media && global_cpu_usage < 15.0 {
            return WorkMode::PassiveStream;
        }

        // По дефолту при активной работе за ПК — неактивно
        WorkMode::ActiveStop
    }
}
