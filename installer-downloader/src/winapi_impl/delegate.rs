//! This module implements [AppDelegate] and [Queue], which allows the NWG UI to be hooked up to our
//! generic controller.

use native_windows_gui::{self as nwg, Event};
use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;

use super::ui::{AppWindow, QUEUE_MESSAGE};
use crate::controller::{AppDelegate, AppDelegateQueue};

impl AppDelegate for AppWindow {
    type Queue = Queue;

    fn on_download<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let button_handle = self.download_button.handle;
        // Register a window message for clicking this button that triggers `callback`.
        nwg::bind_event_handler(
            &button_handle,
            &self.window.handle,
            move |evt, _, handle| {
                if evt == Event::OnButtonClick && handle == button_handle {
                    callback();
                }
            },
        );
    }

    fn on_cancel<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        // TODO
    }

    fn set_status_text(&mut self, text: &str) {
        // TODO
    }

    fn show_download_progress(&mut self) {
        self.progress_bar.set_visible(true);
    }

    fn hide_download_progress(&mut self) {
        self.progress_bar.set_visible(false);
    }

    fn set_download_progress(&mut self, complete: u32) {
        self.progress_bar.set_pos(complete);
    }

    fn show_download_button(&mut self) {
        self.download_button.set_visible(true);
    }

    fn hide_download_button(&mut self) {
        self.download_button.set_visible(false);
    }

    fn enable_download_button(&mut self) {
        self.download_button.set_enabled(true);
    }

    fn disable_download_button(&mut self) {
        self.download_button.set_enabled(false);
    }

    fn show_cancel_button(&mut self) {
        // TODO
    }

    fn hide_cancel_button(&mut self) {
        // TODO
    }

    fn queue(&self) -> Self::Queue {
        Queue {
            main_wnd: self.window.handle,
        }
    }
}

/// Queue sends a window message to the main window containing a [QueueContext], giving us mutable
/// access to the [AppDelegate] on the main UI thread.
///
/// See [QueueContext] docs for more information.
#[derive(Clone)]
pub struct Queue {
    main_wnd: nwg::ControlHandle,
}

// SAFETY: It is safe to post window messages across threads
unsafe impl Send for Queue {}

/// The context contains a callback function that is passed as a pointer to the main thread
/// along with a custom window message `QUEUE_MESSAGE`.
///
/// It must be wrapped in a struct since we cannot pass a fat pointer
/// `*mut dyn for<'a> FnOnce(&'a mut AppWindow) + Send` to `PostMessageW`.
pub struct QueueContext {
    pub callback: Box<dyn for<'a> FnOnce(&'a mut AppWindow) + Send>,
}

impl AppDelegateQueue<AppWindow> for Queue {
    fn queue_main<F: FnOnce(&mut AppWindow) + 'static + Send>(&self, callback: F) {
        let Some(hwnd) = self.main_wnd.hwnd() else {
            return;
        };
        let context = QueueContext {
            callback: Box::new(callback),
        };
        let context_ptr = Box::into_raw(Box::new(context));
        // SAFETY: This is safe since `callback` is Send
        unsafe { PostMessageW(hwnd as _, QUEUE_MESSAGE, 0, context_ptr as isize) };
    }
}
