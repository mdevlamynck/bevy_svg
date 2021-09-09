//! Contains the plugin and its helper types.
//!
//! The [`SvgBundle`](bundle::SvgBundle) provides the creation of shapes with
//! minimal boilerplate.
//!
//! ## How it works
//! The user spawns a [`SvgBundle`](crate::bundle::SvgBundle) from a
//! system in the [`Update`](bevy::app::CoreStage::Update) stage.
//!
//! Then, in the [`SVG`](Stage::SVG) stage, there is a system
//! that associates the corresponding mesh for each entity that has been
//! spawned as a `SvgBundle`.

use bevy::prelude::*;
use lyon_tessellation::{FillTessellator, StrokeTessellator};

use crate::{bundle, loader, render, svg};

/// Stages for this plugin.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum Stage {
    /// Stage in which [`SvgBundle`](crate::bundle::SvgBundle)s get converted
    /// into drawable meshes.
    SVG,
}

/// A plugin that provides resources and a system to draw [`SvgBundle`]s in Bevy
/// with.
pub struct SvgPlugin;

impl Plugin for SvgPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // Loading
        app.add_asset::<svg::Svg>()
            .init_asset_loader::<loader::SvgLoader>()
            .insert_resource(loader::SvgMeshMap::default())
            .insert_resource(FillTessellator::new())
            .insert_resource(StrokeTessellator::new())
            .add_system(loader::svg_mesh_generator.system());

        // Rendering
        app.add_startup_system(render::setup.system());

        // Updating Bundle
        app.add_stage_after(
            bevy::app::CoreStage::Update,
            Stage::SVG,
            SystemStage::parallel(),
        )
        .add_system_to_stage(Stage::SVG, bundle::attach_mesh.system());
    }
}
