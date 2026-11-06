use objc2::rc::Retained;
use objc2_app_kit::{NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
use objc2_core_graphics::{CGWindowLevelForKey, CGWindowLevelKey};
use petal::rwh::{HasWindowHandle, RawWindowHandle};
use winit::platform::macos::WindowAttributesExtMacOS;

use winit::window::{Window, WindowAttributes};
pub(super) fn cfg_window_attribute(window_attrib: WindowAttributes) -> WindowAttributes {
    window_attrib
        .with_title_hidden(true)
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
}

fn ns_window_for_window(window: &Window) -> Retained<NSWindow> {
    let handle = window.window_handle().unwrap().as_raw();
    let handle = match handle {
        RawWindowHandle::AppKit(handle) => handle,
        _ => panic!("Expected AppKit window handle"),
    };

    let view = unsafe { Retained::retain(handle.ns_view.as_ptr() as *mut NSView).unwrap() };
    view.window().unwrap()
}

pub(super) fn cfg_window(window: &Window) {
    let ns_window = ns_window_for_window(window);
    ns_window.setStyleMask(NSWindowStyleMask::Borderless);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    ns_window.setLevel(CGWindowLevelForKey(CGWindowLevelKey::DesktopWindowLevelKey) as isize);
    ns_window.setIgnoresMouseEvents(true);
}
