use std::sync::atomic::{self, AtomicBool};

static IS_BACKTRACE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_backtrace(is_enabled: bool) {
    IS_BACKTRACE_ENABLED.store(is_enabled, atomic::Ordering::SeqCst);
}

pub fn exit<E>(code: i32, e: E) -> !
where
    E: std::error::Error,
{
    eprintln!("Fatal error: {}", e);

    let is_enabled = IS_BACKTRACE_ENABLED.load(atomic::Ordering::SeqCst);
    if is_enabled {
        let trace = backtrace::Backtrace::new();
        eprintln!("\nBacktrace:\n{:?}", trace);
    }

    std::process::exit(code)
}
