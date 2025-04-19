// src/utils/logger.rs
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use chrono::Local;
use once_cell::sync::Lazy;

pub static LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_dir = "logs";
    fs::create_dir_all(log_dir).expect("не удалось создать папку для логов");
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("{}/app_{}.log", log_dir, timestamp);
    let logger = Logger::new(&filename);
    logger.rotate_logs(log_dir, 20);
    logger
});

pub struct Logger {
    enabled: Mutex<bool>,
    file: Mutex<fs::File>,
}

impl Logger {
    fn new(filename: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .expect("не удалось открыть файл для логов");

        Logger {
            enabled: Mutex::new(true),
            file: Mutex::new(file),
        }
    }

    fn rotate_logs(&self, log_dir: &str, max_files: usize) {
        let mut log_files: Vec<_> = fs::read_dir(log_dir)
            .expect("не удалось прочитать папку логов")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|ext| ext == "log").unwrap_or(false) {
                    Some((path, entry.metadata().ok()?.modified().ok()?))
                } else {
                    None
                }
            })
            .collect();

        log_files.sort_by(|a, b| a.1.cmp(&b.1));

        while log_files.len() > max_files {
            let (path, _) = log_files.remove(0);
            fs::remove_file(&path).expect("не удалось удалить старый лог");
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut enabled_guard = self.enabled.lock().expect("не удалось заблокировать флаг");
        *enabled_guard = enabled;
    }

    pub fn log(&self, message: &str, file: &str, line: u32) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_message = format!("[{}] {}:{} - {}\n", timestamp, file, line, message);

        {
            let mut file_guard = self.file.lock().expect("не удалось заблокировать файл");
            file_guard.write_all(log_message.as_bytes()).expect("ошибка записи в файл");
            file_guard.flush().expect("ошибка сброса буфера");
        }

        let enabled = *self.enabled.lock().expect("не удалось заблокировать флаг");
        if enabled {
            print!("{}", log_message);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        crate::utils::logger::LOGGER.log($msg, file!(), line!())
    };
}