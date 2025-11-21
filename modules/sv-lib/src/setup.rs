use std::sync::{Arc, atomic::AtomicBool};
use std::sync::atomic::Ordering;
use colored::Colorize;
use ctrlc;

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
