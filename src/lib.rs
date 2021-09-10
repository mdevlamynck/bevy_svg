//! Load and display simple SVG files in Bevy.
//!
//! This crate provides a Bevy [`Plugin`] to easily load and display a simple
//! SVG file. It currently only works for the most simple SVGs.
//!
//! ## Usage
//! Simply add the crate in your `Cargo.toml` and add the plugin to your app:
//!
//! ```no_run
//! # use bevy::prelude::*;
//! fn main() {
//!     App::build().add_plugin(bevy_svg::SvgPlugin).run();
//! }
//! ```
//!
//! You can now create an entity rendered with a svg:
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_svg::*;
//! fn spawn_svp_entity(mut commands: Commands, asset_server: ResMut<AssetServer>) {
//!     commands.spawn_bundle(SvgBundle::build(SvgBundleConfig {
//!         svg:      asset_server.load("some.svg"),
//!         position: Vec3::new(0.0, 0.0, 0.0),
//!         scale:    Vec2::new(1.0, 1.0),
//!     }));
//! }
//! ```

// rustc
#![deny(future_incompatible, nonstandard_style)]
#![warn(missing_docs, rust_2018_idioms, unused)]
#![allow(elided_lifetimes_in_paths)]
// clippy
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use bevy::prelude::*;

mod bundle;
mod loader;
mod plugin;
mod render;
mod svg;
mod vertex_buffer;

pub use crate::{
    bundle::{SvgBundle, SvgBundleConfig},
    plugin::SvgPlugin,
};
