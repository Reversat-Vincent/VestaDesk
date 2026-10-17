//! The user's preferences ([`Settings`]), saved in the settings directory as the [`Document`] `settings.json`.
//!
//! The app loads them at startup and saves them each time they change, with [`load_and_autosave`].
//! A settings file that cannot be loaded is never overwritten, so the user can still repair it.

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::router::Route;
use crate::storage::{self, AutoSave, Document};

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
    /// The page open when the app last closed, which it reopens on, or the Home page if this version cannot read the saved page.
    #[serde(deserialize_with = "or_default")]
    pub page: Route,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            page: Route::default(),
        }
    }
}

impl Document for Settings {
    const FILE_NAME: &'static str = "settings.json";
    const VERSION: u32 = 1;

    fn correct(&mut self) {
        // A zoom that is not a number has no nearest limit, so it takes the default.
        self.zoom = if self.zoom.is_nan() {
            Self::default().zoom
        } else {
            self.zoom.clamp(MIN_ZOOM, MAX_ZOOM)
        };
    }
}

/// Returns the settings saved in `dir`, and the [`AutoSave`] that saves their changes there.
///
/// If the settings file cannot be loaded, it reports the error and returns the default settings
/// without an [`AutoSave`], so that the file is not overwritten. If the [`AutoSave`] cannot start,
/// it reports the error too, and the settings are kept in memory only.
#[must_use]
pub fn load_and_autosave(dir: PathBuf) -> (Settings, Option<AutoSave<Settings>>) {
    match storage::load::<Settings>(&dir) {
        Ok(settings) => {
            let autosave = AutoSave::start(settings.clone(), move |settings| {
                storage::save(&dir, settings)
            })
            .inspect_err(AppError::report)
            .ok();
            (settings, autosave)
        }
        Err(error) => {
            error.report();
            (Settings::default(), None)
        }
    }
}

/// Deserializes a value, or gives its default value if this version of VestaDesk cannot read it, such as a page that only a newer version has.
fn or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}
