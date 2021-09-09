//! Bevy [`Bundle`] representing an SVG entity.

use bevy::{
    prelude::*,
    render::{
        pipeline::{RenderPipeline, RenderPipelines},
        render_graph::base::MainPass,
    },
};

use crate::{loader::SvgMeshMap, render, svg::Svg};

/// A Bevy [`Bundle`] representing an SVG entity.
#[allow(missing_docs)]
#[derive(Bundle)]
pub struct SvgBundle {
    pub svg:              Handle<Svg>,
    pub need_mesh_update: NeedMeshUpdate,
    pub mesh:             Handle<Mesh>,
    pub main_pass:        MainPass,
    pub draw:             Draw,
    pub visible:          Visible,
    pub render_pipelines: RenderPipelines,
    pub transform:        Transform,
    pub global_transform: GlobalTransform,
}

impl SvgBundle {
    /// Create a new [`SvgBundle`] from a [`SvgBundleConfig`].
    pub fn build(config: SvgBundleConfig) -> SvgBundle {
        Self {
            svg:              config.svg,
            need_mesh_update: Default::default(),
            mesh:             Default::default(),
            main_pass:        MainPass,
            draw:             Default::default(),
            visible:          Visible {
                is_visible:     false,
                is_transparent: true,
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                render::SVG_PIPELINE_HANDLE.typed(),
            )]),
            transform:        Transform {
                translation: config.position,
                scale: Vec3::new(1.0, -1.0, 1.0) * config.scale.extend(1.0),
                ..Default::default()
            },
            global_transform: Default::default(),
        }
    }
}

/// Config to build a [`SvgBundle`].
pub struct SvgBundleConfig {
    /// SVG to render.
    pub svg:      Handle<Svg>,
    /// Position at which the [`SvgBundle`] will be spawned in Bevy.
    /// The center of the SVG will be at this position.will be at this position.
    pub position: Vec3,
    /// Value by which the SVG will be scaled, default is (1.0, 1.0).
    pub scale:    Vec2,
}

impl Default for SvgBundleConfig {
    fn default() -> Self {
        Self {
            svg:      Default::default(),
            position: Vec3::new(0., 0., 0.),
            scale:    Vec2::new(1.0, 1.0),
        }
    }
}

#[derive(Default)]
pub struct NeedMeshUpdate;

pub fn attach_mesh(
    mut commands: Commands,
    svg_mesh_map: ResMut<SvgMeshMap>,
    mut query: Query<(Entity, &Handle<Svg>, &mut Handle<Mesh>, &mut Visible), With<NeedMeshUpdate>>,
) {
    for (entity, svg, mut mesh, mut visible) in query.iter_mut() {
        if let Some(handle) = svg_mesh_map.0.get(svg) {
            *mesh = handle.clone();
            visible.is_visible = true;

            commands.entity(entity).remove::<NeedMeshUpdate>();
        }
    }
}
