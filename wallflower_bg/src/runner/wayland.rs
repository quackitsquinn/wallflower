use std::println;
use std::ptr::NonNull;

use anyhow::Context;
use petal::rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};

use petal::w;
use smithay_client_toolkit::{
    compositor::{CompositorState, FrameCallbackData},
    delegate_registry,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface},
    },
};
use wayland_client::{Connection, Proxy, QueueHandle, globals::registry_queue_init};

use crate::Wallflower;

mod handlers;

// The main function of our program
pub fn run_wayland() -> anyhow::Result<()> {
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) =
        registry_queue_init(&conn).with_context(|| "registry_queue_init failed")?;
    let qh: QueueHandle<WaylandRunner> = event_queue.handle();
    let layer_shell = LayerShell::bind(&globals, &qh).with_context(|| "LayerShell::bind failed")?;
    let compositor =
        CompositorState::bind(&globals, &qh).with_context(|| "CompositorState::bind failed")?;

    let surf = create_surface(&qh, &compositor, &layer_shell);
    let wallflower = init_wallflower(&conn, &surf).with_context(|| "failed to init renderer")?;

    let mut state = WaylandRunner {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        surface: surf,
        wallflower,
        should_close: false,
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

fn create_surface(
    qh: &QueueHandle<WaylandRunner>,
    compositor: &CompositorState,
    layer_shell: &LayerShell,
) -> LayerSurface {
    let surf = compositor.create_surface(qh);
    let surf = layer_shell.create_layer_surface(
        qh,
        surf.clone(),
        Layer::Background,
        Some("wallflower"),
        None,
    );

    surf.set_anchor(Anchor::TOP | Anchor::LEFT);
    surf.set_keyboard_interactivity(KeyboardInteractivity::None);
    let size = (1000, 1000);
    surf.set_size(size.0, size.1);

    surf.commit();

    surf
}

fn init_wallflower(conn: &Connection, surf: &LayerSurface) -> anyhow::Result<Wallflower> {
    let target = w::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr().cast()).unwrap(),
        ))),
        raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surf.wl_surface().id().as_ptr().cast()).unwrap(),
        )),
    };

    unsafe { Wallflower::new_unsafe_surface(target) }
}

struct WaylandRunner {
    registry_state: RegistryState,
    output_state: OutputState,
    should_close: bool,
    wallflower: Wallflower,
    surface: LayerSurface,
}

impl WaylandRunner {
    fn render(&mut self, qh: &QueueHandle<Self>) {
        self.request_redraw(qh);
        self.wallflower.render();
        self.surface.commit();
    }

    fn request_redraw(&mut self, qh: &QueueHandle<Self>) {
        self.surface
            .wl_surface()
            .frame(qh, FrameCallbackData(self.surface.wl_surface().clone()));
    }
}

delegate_registry!(WaylandRunner);

impl ProvidesRegistryState for WaylandRunner {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

smithay_client_toolkit::delegate_dispatch2!(WaylandRunner);
