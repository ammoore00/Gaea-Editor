use std::{env, panic};
use std::sync::{Mutex, Once};
use std::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref HOOK_INIT: Once = Once::new();
    static ref ORIGINAL_HOOK: Mutex<Option<Box<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static>>> = Mutex::new(None);
    static ref EXPECTED_PANICS: AtomicUsize = AtomicUsize::new(0);
    static ref OCCURRED_PANICS: AtomicUsize = AtomicUsize::new(0);
}

pub struct PanicSilencer {
    expected_panic_count: usize,
}

impl PanicSilencer {
    pub fn new(expected_panic_count: usize) -> Self {
        // Store previous environment variable and set to suppress backtraces
        unsafe {
            let _ = env::var("RUST_BACKTRACE").map(|_| env::set_var("RUST_BACKTRACE", "0"));
        }

        // Initialize the custom panic hook exactly once
        HOOK_INIT.call_once(|| {
            // Store the original hook
            let original_hook = panic::take_hook();
            *ORIGINAL_HOOK.lock().unwrap() = Some(original_hook);

            // Install our custom hook that counts panics
            panic::set_hook(Box::new(|info| {
                let occurred = OCCURRED_PANICS.fetch_add(1, Ordering::SeqCst) + 1;
                let expected = EXPECTED_PANICS.load(Ordering::SeqCst);

                // Only show panic information if we exceed expected panics
                if occurred > expected {
                    if let Some(loc) = info.location() {
                        eprintln!("panic occurred at {}:{}: {}", loc.file(), loc.line(),
                                  info.payload().downcast_ref::<&str>().unwrap_or(&"<unknown panic message>"));
                    } else {
                        eprintln!("panic occurred: {}",
                                  info.payload().downcast_ref::<&str>().unwrap_or(&"<unknown panic message>"));
                    }
                }
            }));
        });

        // Add this silencer's expected panics to the global count
        EXPECTED_PANICS.fetch_add(expected_panic_count, Ordering::SeqCst);

        Self { expected_panic_count }
    }
}

impl Drop for PanicSilencer {
    fn drop(&mut self) {
        // Decrement the expected panic count
        // This is safe even during a panic
        let current = EXPECTED_PANICS.load(Ordering::SeqCst);
        let new_val = current.saturating_sub(self.expected_panic_count);
        EXPECTED_PANICS.store(new_val, Ordering::SeqCst);

        // We never restore the original hook because it could be called during a panic
        // Instead, we'll let the process exit normally, which is safer
    }
}