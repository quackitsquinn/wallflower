use crate::{Layer, LoadedImage};

pub struct LayerCompositor {
    layers: Vec<Layer>,
}

impl LayerCompositor {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }
}
