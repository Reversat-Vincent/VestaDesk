//! The user's preferences ([`Settings`]), saved in the settings directory as the [`Document`] `settings.json`.

use serde::{Deserialize, Serialize};

use crate::storage::Document;

/// The smallest zoom factor: 50 %.
pub const MIN_ZOOM: f32 = 0.5;

/// The largest zoom factor: 300 %.
pub const MAX_ZOOM: f32 = 3.0;

/// The user's preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The zoom factor of the whole interface, from [`MIN_ZOOM`] to [`MAX_ZOOM`]; 1.0 is 100 %.
    pub zoom: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

impl Document for Settings {
    const FILE_NAME: &'static str = "settings.json";
    const VERSION: u32 = 1;

    fn correct(&mut self) {
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }
}
