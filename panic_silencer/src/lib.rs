use std::panic;
use std::sync::{Mutex, Once};
use std::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref HOOK_INIT: Once = Once::new();
    static ref ORIGINAL_HOOK: Mutex<Option<Box<dyn Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static>>> = Mutex::new(None);
    static ref EXPECTED_PANICS: AtomicUsize = AtomicUsize::new(0);
    static ref OCCURRED_PANICS: AtomicUsize = AtomicUsize::new(0);
}

pub struct PanicSilencer;

impl PanicSilencer {
    pub fn new(expected_panic_count: usize) -> Self {
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
                    ORIGINAL_HOOK.lock().unwrap().as_ref().map(|hook| hook(info));
                }
            }));
        });

        // Add this silencer's expected panics to the global count
        EXPECTED_PANICS.fetch_add(expected_panic_count, Ordering::SeqCst);
        Self
    }
}