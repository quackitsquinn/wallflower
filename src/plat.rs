use winit::window::{Window, WindowAttributes};

#[cfg(target_os = "macos")]
fn cfg_window_attribute_macos(window_attrib: WindowAttributes) -> WindowAttributes {
    use winit::platform::macos::WindowAttributesExtMacOS;

    window_attrib
        .with_title_hidden(true)
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
}

#[cfg(target_os = "macos")]
fn cfg_window_macos(window: &Window) {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSView, NSWindowCollectionBehavior, NSWindowStyleMask};
    use objc2_core_graphics::{CGWindowLevelForKey, CGWindowLevelKey};
    use w::rwh::{HasWindowHandle, RawWindowHandle};

    let handle = window.window_handle().unwrap().as_raw();
    let handle = match handle {
        RawWindowHandle::AppKit(handle) => handle,
        _ => panic!("Expected AppKit window handle"),
    };

    let view = unsafe { Retained::retain(handle.ns_view.as_ptr() as *mut NSView).unwrap() };
    let ns_window = view.window().unwrap();
    ns_window.setStyleMask(NSWindowStyleMask::Borderless);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    ns_window.setLevel(CGWindowLevelForKey(CGWindowLevelKey::DesktopWindowLevelKey) as isize);
    ns_window.setIgnoresMouseEvents(true);
}

pub fn cfg_window_platform(window: &Window) {
    #[cfg(target_os = "macos")]
    return cfg_window_macos(window);

    #[cfg(not(target_os = "macos"))]
    return panic!("cfg_window_platform is only implemented for macOS");
}

pub fn cfg_window_attrib_platform(window_attrib: WindowAttributes) -> WindowAttributes {
    #[cfg(target_os = "macos")]
    return cfg_window_attribute_macos(window_attrib);

    #[cfg(not(target_os = "macos"))]
    return panic!("cfg_window_platform is only implemented for macOS");
}
