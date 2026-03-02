use console::style;
use chrono::Local;

pub fn get_timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

pub fn info(message: &str) {
    let timestamp = get_timestamp();
    println!("{} {} {}", style(format!("[{}]", timestamp)).green(), style("[INFO]").bold(), message);
    log_write(&format!("[{}] [INFO] {}", timestamp, message));
}

pub fn warn(message: &str) {
    let timestamp = get_timestamp();
    eprintln!("{} {} {}", style(format!("[{}]", timestamp)).green(), style("[WARN]").bold().yellow(), message);
    log_write(&format!("[{}] [WARN] {}", timestamp, message));
}

pub fn error(message: &str) {
    let timestamp = get_timestamp();
    eprintln!("{} {} {}", style(format!("[{}]", timestamp)).green(), style("[ERROR]").bold().red(), message);
    log_write(&format!("[{}] [ERROR] {}", timestamp, message));
}

use std::sync::OnceLock;

static LOG_FILE: OnceLock<String> = OnceLock::new();

pub fn init() {
    let log_dir = "logs";
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        eprintln!("{} Failed to create logs directory: {}", style("[ERROR]").bold().red(), e);
        return;
    }
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_path = format!("{}/noodle_{}.txt", log_dir, timestamp);
    
    if let Err(e) = std::fs::File::create(&log_path) {
        eprintln!("{} Failed to create log file: {}", style("[ERROR]").bold().red(), e);
        return;
    }
    
    LOG_FILE.set(log_path).ok();
}

fn log_write(message: &str) {
    if let Some(log_path) = LOG_FILE.get() {
        if let Err(e) = std::fs::OpenOptions::new().append(true).open(log_path).and_then(|mut file| {
            use std::io::Write;
            writeln!(file, "{}", message)
        }) {
            eprintln!("{} Failed to write to log file: {}", style("[ERROR]").bold().red(), e);
        }
    }
}