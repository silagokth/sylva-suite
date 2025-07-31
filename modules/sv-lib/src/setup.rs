use std::sync::{Arc, atomic::AtomicBool};
use std::sync::atomic::Ordering;
use ctrlc;

pub fn interrupt() -> Result<Arc<AtomicBool>, Box<dyn std::error::Error>> {
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
    })?;

    Ok(interrupted)
}
