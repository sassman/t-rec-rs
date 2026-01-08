//! Snake_case method wrappers for `CALayer`.

use objc2_quartz_core::CALayer;

/// Extension trait providing snake_case methods for CALayer.
pub trait CALayerExt {
    /// Add a sublayer to this layer.
    fn add_sublayer(&self, layer: &CALayer);
}

impl CALayerExt for CALayer {
    fn add_sublayer(&self, layer: &CALayer) {
        self.addSublayer(layer);
    }
}
