use winit::window::{Window, WindowAttributes};

#[cfg(target_os = "macos")]
mod macos;

pub fn cfg_window_platform(window: &Window) {
    #[cfg(target_os = "macos")]
    return macos::cfg_window(window);
}

pub fn cfg_window_attrib_platform(window_attrib: WindowAttributes) -> WindowAttributes {
    #[cfg(target_os = "macos")]
    return macos::cfg_window_attribute(window_attrib);

    #[cfg(not(target_os = "macos"))]
    window_attrib
}
