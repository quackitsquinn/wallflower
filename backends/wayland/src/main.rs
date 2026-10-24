use std::{ptr::NonNull, todo};

use anyhow::Context;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler},
    },
};
use w::rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, globals::registry_queue_init, protocol::wl_registry,
};

// The main function of our program
fn main() -> anyhow::Result<()> {
    env_logger::init();
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) =
        registry_queue_init(&conn).with_context(|| "registry_queue_init failed")?;
    let qh: QueueHandle<State> = event_queue.handle();
    let compositor =
        CompositorState::bind(&globals, &qh).with_context(|| "CompositorState::bind failed")?;
    let layer_shell = LayerShell::bind(&globals, &qh).with_context(|| "LayerShell::bind failed")?;
    let surf = compositor.create_surface(&qh);

    let surf = layer_shell.create_layer_surface(
        &qh,
        surf.clone(),
        Layer::Background,
        Some("quackits.quinn.wallflower"),
        None,
    );

    surf.set_anchor(Anchor::TOP | Anchor::LEFT);
    surf.set_keyboard_interactivity(KeyboardInteractivity::None);
    let size = (1000, 1000);
    surf.set_size(size.0, size.1);

    surf.commit();

    let target = w::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr().cast()).unwrap(),
        ))),
        raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surf.wl_surface().id().as_ptr().cast()).unwrap(),
        )),
    };

    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        should_close: false,
        gfx: unsafe { petal::init_graphics_raw(target, size) }
            .with_context(|| "failed to initialize graphics")?,
    };

    loop {
        event_queue
            .blocking_dispatch(&mut state)
            .with_context(|| "blocking_dispatch failed")?;

        if state.should_close {
            break;
        }
    }

    Ok(())
}

struct State {
    registry_state: RegistryState,
    output_state: OutputState,
    should_close: bool,
    gfx: petal::Graphics,
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for State {
    fn closed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
        self.should_close = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        self.gfx.render_frame().unwrap()
    }
}

delegate_registry!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

smithay_client_toolkit::delegate_dispatch2!(State);
