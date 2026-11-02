//! Ownership of the operating-system wake request.
//!
//! The request is thread-scoped, so it must be issued from the thread that runs
//! the message loop. The guard exists so that every exit path - normal, error,
//! or panic - releases the request. Nothing else in the program is allowed to
//! call `SetThreadExecutionState`.

use windows_sys::Win32::Foundation::{GetLastError, SetLastError};
use windows_sys::Win32::System::Power::SetThreadExecutionState;

const ES_CONTINUOUS: u32 = 0x8000_0000;
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;

pub struct PowerGuard {
    held: bool,
    last_error: u32,
}

impl PowerGuard {
    pub const fn new() -> Self {
        Self {
            held: false,
            last_error: 0,
        }
    }

    pub fn is_held(&self) -> bool {
        self.held
    }

    pub fn last_error(&self) -> u32 {
        self.last_error
    }

    /// Ask the OS to keep both the system and the display awake.
    ///
    /// The return value of `SetThreadExecutionState` is the *previous* state,
    /// which is legitimately zero on a thread that never set one, so success is
    /// decided by the thread's last-error value instead.
    pub fn acquire(&mut self) -> bool {
        // SAFETY: plain Win32 call with constant flags.
        unsafe {
            SetLastError(0);
            let _previous = SetThreadExecutionState(
                ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
            );
            self.last_error = GetLastError();
        }
        self.held = self.last_error == 0;
        self.held
    }

    /// Drop back to the OS defaults for this thread.
    pub fn release(&mut self) {
        if !self.held {
            return;
        }
        // SAFETY: clearing with ES_CONTINUOUS only is the documented reset.
        unsafe {
            SetLastError(0);
            SetThreadExecutionState(ES_CONTINUOUS);
            self.last_error = GetLastError();
        }
        self.held = false;
    }
}

impl Drop for PowerGuard {
    fn drop(&mut self) {
        self.release();
    }
}
