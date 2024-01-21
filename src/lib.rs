#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(missing_docs)]

//! # Pulse
//!
//! ![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
//!
//! ## What is Pulse?
//!
//! Pulse is an ambitious game engine in the **very** early stages of development (it's pretty empty
//! here right now). Planned features include:
//! - Render graphs for building custom rendering pipeline and effects
//! - Physically based rendering
//! - Image based lighting
//! - Custom material system
//! - Scripting engine
//! - Integrating with existing physics libraries
//! - UI
//! - Editor
//! - Asset management system
//! - Mouse, keyboard, and gamepad input

pub use crate::app::Application;
pub use crate::app::ApplicationState;
pub use crate::app::Event;
pub use crate::components::ComputedVisibility;
pub use crate::components::LocalTransform;
pub use crate::components::Visibility;
pub use crate::scene::Component;
pub use crate::scene::ComponentEvent;
pub use crate::scene::Node;
pub use crate::scene::Scene;

mod app;
mod components;
mod scene;
pub mod systems;
