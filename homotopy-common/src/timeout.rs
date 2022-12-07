use std::sync::atomic::{AtomicBool, Ordering};

static SHOULD_TIMEOUT: AtomicBool = AtomicBool::new(false);

/// Call this anywhere in the codebase
/// If it returns error, raise it with the ?-operator.
#[allow(clippy::result_unit_err)]
pub fn timeout_if_needed() -> Result<(), ()> {
    if SHOULD_TIMEOUT.load(Ordering::Relaxed) {
        reset_timeout_state();
        Err(())
    } else {
        Ok(())
    }
}

/// Map this to timeout buttons to ask the tool
/// to quit any heavy calculation at the first chance.
pub fn timeout_now() {
    SHOULD_TIMEOUT.store(true, Ordering::Relaxed);
}

/// Call this at the end of a refresh cycle.
pub fn reset_timeout_state() {
    SHOULD_TIMEOUT.store(false, Ordering::Relaxed);
}
