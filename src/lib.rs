use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, Position},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowButtons, WindowId, WindowLevel},
};

pub struct Wallflower {
    window: Option<Window>,
}

impl Default for Wallflower {
    fn default() -> Self {
        Self { window: None }
    }
}

fn background_window_attribs(el: &ActiveEventLoop) -> WindowAttributes {
    let mut wa = WindowAttributes::default()
        .with_position(PhysicalPosition::new(0, 0))
        .with_window_level(WindowLevel::AlwaysOnBottom)
        .with_resizable(false);

    let monitor = el.primary_monitor().unwrap();
    let size = monitor.size();
    wa = wa.with_inner_size(size);

    #[cfg(target_os = "macos")]
    {
        wa = cfg_window_attribute_macos(wa)
    }

    wa
}

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
    use std::mem;

    use objc2::rc::Retained;
    use objc2_app_kit::{NSView, NSWindowCollectionBehavior, NSWindowStyleMask};
    use objc2_core_graphics::{CGWindowLevelForKey, CGWindowLevelKey};
    use wgpu::rwh::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};

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

impl ApplicationHandler for Wallflower {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(background_window_attribs(event_loop))
            .unwrap();
        cfg_window_macos(&window);
        window.set_outer_position(PhysicalPosition::new(0, 0));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Exiting!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
