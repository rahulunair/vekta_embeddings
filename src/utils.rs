use std::env;
use sysinfo::{System, SystemExt};

pub fn detect_system_resources() -> usize {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    let cpu_count = system.physical_core_count().unwrap_or(1);

    // Determine batch size based on available memory
    let batch_size = if total_memory < 4 * 1024 * 1024 * 1024 {
        // Less than 4GB RAM
        1
    } else if total_memory < 8 * 1024 * 1024 * 1024 {
        // 4-8GB RAM
        4
    } else if total_memory < 16 * 1024 * 1024 * 1024 {
        // 8-16GB RAM
        8
    } else {
        16 // More than 16GB RAM
    };

    log(&format!(
        "Detected system: {} cores, {} GB RAM",
        cpu_count,
        total_memory / (1024 * 1024 * 1024)
    ));
    log(&format!("Using batch size: {}", batch_size));

    batch_size
}

pub fn log(message: &str) {
    if env::var("VEKTA_QUIET").unwrap_or_default() != "1" {
        eprintln!("{}", message);
    }
}
