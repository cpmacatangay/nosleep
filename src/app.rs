//! Wiring: state machine ↔ power request ↔ tray icon.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    KillTimer, SetTimer, WM_CONTEXTMENU, WM_LBUTTONUP, WM_RBUTTONUP,
};

use crate::icon::{self, HIcon, ASLEEP_RGB, AWAKE_RGB};
use crate::power::PowerGuard;
use crate::state::State;
use crate::tray;

/// Version-4 tray notifications. Defined here so the intent is readable next to
/// the handling code rather than hidden behind a bindgen name.
const NIN_SELECT: u32 = 0x0400;
const NIN_KEYSELECT: u32 = 0x0401;

/// Adding a tray icon can fail if Explorer's notification area is not ready yet,
/// which happens on a slow sign-in. Retry briefly rather than giving up.
const RETRY_TIMER: usize = 1;
const RETRY_INTERVAL_MS: u32 = 2000;
const RETRY_LIMIT: u32 = 15;

/// What the window procedure should do after a tray callback.
pub enum Action {
    None,
    Toggle,
    ShowMenu,
}

pub struct App {
    pub state: State,
    hwnd: HWND,
    callback: u32,
    icon_on: HIcon,
    icon_off: HIcon,
    version4: bool,
    tray_added: bool,
    retries: u32,
    power: PowerGuard,
    closed: bool,
}

impl App {
    pub fn new() -> Self {
        // Render at the shell's real small-icon size so the artwork stays crisp
        // at any display scaling.
        let size = icon::tray_size();
        Self {
            state: State::START,
            hwnd: null_mut(),
            callback: 0,
            icon_on: icon::render(size, icon::Style::Disc, AWAKE_RGB),
            icon_off: icon::render(size, icon::Style::Ring, ASLEEP_RGB),
            version4: false,
            tray_added: false,
            retries: 0,
            power: PowerGuard::new(),
            closed: false,
        }
    }

    pub fn attach(&mut self, hwnd: HWND, callback: u32) {
        self.hwnd = hwnd;
        self.callback = callback;
        self.try_add();
        if !self.tray_added {
            self.schedule_retry();
        }
    }

    /// Re-register after Explorer restarts and broadcasts `TaskbarCreated`.
    pub fn re_add(&mut self) {
        if self.closed {
            return;
        }
        self.try_add();
        if !self.tray_added {
            self.schedule_retry();
        }
    }

    fn try_add(&mut self) {
        self.tray_added = tray::add(
            self.hwnd,
            self.callback,
            self.current_icon(),
            self.state.tooltip(),
        );
        if self.tray_added {
            self.version4 = tray::set_version(self.hwnd, self.callback);
            // Publish the tooltip again, now that version-4 tooltips are active.
            self.refresh();
        }
    }

    fn schedule_retry(&self) {
        // SAFETY: the window handle is live; the timer posts WM_TIMER to it.
        unsafe {
            SetTimer(self.hwnd, RETRY_TIMER, RETRY_INTERVAL_MS, None);
        }
    }

    /// Called from `WM_TIMER`. Stops itself once the icon is up or attempts run out.
    pub fn on_timer(&mut self, timer_id: usize) {
        if timer_id != RETRY_TIMER {
            return;
        }
        if self.closed || self.tray_added {
            self.cancel_retry();
            return;
        }
        self.retries += 1;
        self.try_add();
        if self.tray_added || self.retries >= RETRY_LIMIT {
            self.cancel_retry();
        }
    }

    fn cancel_retry(&self) {
        // SAFETY: killing a timer that may not exist is harmless.
        unsafe {
            KillTimer(self.hwnd, RETRY_TIMER);
        }
    }

    pub fn on_tray_event(&self, event: u32) -> Action {
        let code = event & 0xFFFF;
        if self.version4 {
            match code {
                NIN_SELECT | NIN_KEYSELECT => Action::Toggle,
                WM_CONTEXTMENU => Action::ShowMenu,
                _ => Action::None,
            }
        } else {
            match code {
                WM_LBUTTONUP => Action::Toggle,
                WM_RBUTTONUP => Action::ShowMenu,
                _ => Action::None,
            }
        }
    }

    pub fn toggle(&mut self) {
        let next = self.state.toggled();
        if next.is_on() {
            if !self.power.acquire() {
                // The OS refused the request. Stay Off rather than display a
                // state the machine is not actually in.
                return;
            }
        } else {
            self.power.release();
        }
        self.state = next;
        self.refresh();
    }

    /// Idempotent: called from Quit, sign-out, shutdown, and window teardown.
    pub fn shutdown(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.cancel_retry();
        self.power.release();
        if self.tray_added {
            tray::remove(self.hwnd, self.callback);
            self.tray_added = false;
        }
    }

    fn current_icon(&self) -> HIcon {
        if self.state.is_on() {
            self.icon_on
        } else {
            self.icon_off
        }
    }

    fn refresh(&self) {
        if self.tray_added {
            tray::update(
                self.hwnd,
                self.callback,
                self.current_icon(),
                self.state.tooltip(),
            );
        }
    }
}
