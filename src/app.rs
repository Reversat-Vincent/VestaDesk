//! The main window and its `eframe::App` implementation ([`App`]): the single state container and the frame lifecycle.
//!
//! # Update cycle
//!
//! [`App`] keeps all of the app's data in one [`AppState`], which only [`AppState::apply`] changes.
//! On each frame, eframe calls two methods of [`App`]:
//!
//! 1. [`logic`](eframe::App::logic) counts the notifications down by the time since the last frame, then applies the [actions](crate::action) waiting in the [`ActionQueue`], in order.
//! 2. [`ui`](eframe::App::ui) lets the screens read the data, draw the window and push the user's actions to the queue.
//! 3. If they pushed any, `ui` asks for another frame, whose `logic` applies them.
//!
//! eframe calls `logic` even while the window is minimized, so the actions of background tasks still take effect.
//! While shown notifications count down, `logic` also asks for a frame when the next one's time is up, so that it goes even if the user does nothing.
//!
//! # Rendering
//!
//! eframe draws the window with OpenGL, through its `glow` renderer, instead of wgpu, its default.
//! For an empty window in release mode on Windows 11, glow gives a 7.1 MB program and a first
//! frame in 120 ms, against 13.8 MB and 630 ms for wgpu.
//! VestaDesk therefore needs OpenGL 2.0 or later.

use std::time::Instant;

use egui::{Context, Vec2};

use crate::action::{self, ActionQueue, ActionSender, AppAction};
use crate::error::{AppError, Result};
use crate::fonts;
use crate::layout;
use crate::state::AppState;
use crate::theme;

/// The app's name, shown as the window title.
const NAME: &str = "VestaDesk";

/// The size of the window when it opens, in points.
pub const WINDOW_SIZE: Vec2 = egui::vec2(800.0, 600.0);

/// The smallest size the window can be resized to, in points.
pub const MIN_WINDOW_SIZE: Vec2 = egui::vec2(640.0, 480.0);

/// The VestaDesk app, which eframe runs in the main window.
#[derive(Debug)]
pub struct App {
    /// All of the app's data.
    state: AppState,
    /// The actions waiting to be applied on the next frame.
    actions: ActionQueue,
    /// The sender that [`App::sender`] gives to background tasks.
    sender: ActionSender,
    /// When the last frame started, from which the next one counts the notifications down.
    last_frame: Instant,
}

impl App {
    /// Returns the app with the default data, for the window whose context is `ctx`.
    ///
    /// The app installs the bundled fonts and the colour schemes in `ctx`, and the actions sent to it ask `ctx` for a screen update.
    #[must_use]
    pub fn new(ctx: Context) -> Self {
        fonts::install(&ctx);
        theme::install(&ctx);
        let (sender, actions) = action::channel(ctx);
        Self {
            state: AppState::default(),
            actions,
            sender,
            last_frame: Instant::now(),
        }
    }

    /// Returns all of the app's data.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui::Context;
    /// use vesta_desk::app::App;
    /// use vesta_desk::state::AppState;
    ///
    /// assert_eq!(App::new(Context::default()).state(), &AppState::default());
    /// ```
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Returns a sender that background tasks use to send actions to the app from any thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui::Context;
    /// use vesta_desk::action::AppAction;
    /// use vesta_desk::app::App;
    /// use vesta_desk::router::Route;
    ///
    /// let app = App::new(Context::default());
    /// app.sender().send(AppAction::Navigate(Route::About))?;
    /// # Ok::<(), vesta_desk::error::AppError>(())
    /// ```
    #[must_use]
    pub fn sender(&self) -> ActionSender {
        self.sender.clone()
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_frame);
        self.last_frame = now;
        self.state.apply(AppAction::CountDown(elapsed));
        for action in self.actions.take() {
            self.state.apply(action);
        }
        if let Some(time_left) = self.state.notifications().next_expiry() {
            ctx.request_repaint_after(time_left);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        layout::show(ui, &self.state, &mut self.actions);
        if !self.actions.pushed().is_empty() {
            // egui redraws by itself after input, but actions from a frame without input (such as a timer) need this to show.
            ui.ctx().request_repaint();
        }
    }
}

/// Returns the options of the main window: its title, size and minimum size, centered on the screen.
///
/// # Examples
///
/// ```
/// use vesta_desk::app::{MIN_WINDOW_SIZE, native_options};
///
/// let window = native_options().viewport;
/// assert_eq!(window.title.as_deref(), Some("VestaDesk"));
/// assert_eq!(window.min_inner_size, Some(MIN_WINDOW_SIZE));
/// ```
#[must_use]
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(NAME)
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),
        centered: true,
        ..Default::default()
    }
}

/// Opens the main window and runs the app until the window is closed.
///
/// # Errors
///
/// Returns [`AppError::Startup`] if the window cannot be created, for example without a display or without OpenGL 2.0.
pub fn run() -> Result<()> {
    eframe::run_native(
        NAME,
        native_options(),
        Box::new(|creation| Ok(Box::new(App::new(creation.egui_ctx.clone())))),
    )
    .map_err(|error| AppError::Startup(error.to_string()))
}
