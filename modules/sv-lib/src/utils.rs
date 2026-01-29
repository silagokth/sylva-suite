use std::sync::{Arc, atomic::AtomicBool};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use colored::Colorize;
use once_cell::sync::Lazy;
use ctrlc;

// Use Mutex so we can safely update them at program start
pub static CPU_LIMIT: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(2)); // default 2 CPUs
pub static MEMORY_LIMIT: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(4_000_000)); // default 4GB in KB

pub fn set_cpu_limit(
    cpu: u64, 
) -> Result<(), Box<dyn std::error::Error>> {

    if cpu > 32 {
        return Err("Error: failed to set resource limit".into());
    }

    *CPU_LIMIT.lock().unwrap() = cpu;

    Ok(())
}

pub fn set_memory_limit(
    memory_gb: u64,
) -> Result<(), Box<dyn std::error::Error>> {

    if memory_gb > 64 {
        return Err("Error: failed to set resource limit".into());

    }

    let memory_kb = memory_gb * 1024 * 1024;
    *MEMORY_LIMIT.lock().unwrap() = memory_kb;

    Ok(())
}

pub fn interrupt() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })?;

    Ok(interrupted)
}

pub fn setup_logger(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            // choose color based on log level
            let level = match record.level() {
                log::Level::Error => record.level().to_string().red(),
                log::Level::Warn  => record.level().to_string().yellow(),
                log::Level::Info  => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().blue(),
                log::Level::Trace => record.level().to_string().magenta(),
            };

            out.finish(format_args!(
                "[{date}][{level}] {message}",
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level = level,
                message = message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(file)?)
        .apply()?;

    Ok(())
}
