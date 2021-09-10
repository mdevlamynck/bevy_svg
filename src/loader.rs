use std::collections::HashMap;

use anyhow::Result;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
};
use lyon_tessellation::{self, FillTessellator, StrokeTessellator};

use crate::svg::Svg;

#[derive(Default)]
pub struct SvgLoader;

impl AssetLoader for SvgLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<Result<()>> {
        Box::pin(async move {
            let mut options = usvg::Options::default();
            options.fontdb.load_system_fonts();
            options.fontdb.load_fonts_dir("./assets");

            let tree = usvg::Tree::from_data(bytes, &options.to_ref())?;
            let svg = svg::parse_svg(tree);

            load_context.set_default_asset(LoadedAsset::new(svg));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}

#[derive(Default)]
pub struct SvgMeshMap(pub HashMap<Handle<Svg>, Handle<Mesh>>);

/// Maintains a mapping from [`Svg`] handle to [`Mesh`] handle.
pub fn svg_mesh_generator(
    mut svg_events: EventReader<AssetEvent<Svg>>,
    svg_store: Res<Assets<Svg>>,
    mut svg_mesh_map: ResMut<SvgMeshMap>,
    mut mesh_store: ResMut<Assets<Mesh>>,
    mut fill_tess: ResMut<FillTessellator>,
    mut stroke_tess: ResMut<StrokeTessellator>,
) {
    for event in svg_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                let svg = svg_store.get(handle).unwrap();
                let mesh = tesselation::generate_mesh(&svg, &mut *fill_tess, &mut *stroke_tess);

                let mesh_handle = mesh_store.add(mesh);
                svg_mesh_map.0.insert(handle.clone_weak(), mesh_handle);
            },
            AssetEvent::Modified { handle } => {
                let mesh = mesh_store
                    .get_mut(svg_mesh_map.0.get(handle).unwrap())
                    .unwrap();

                let svg = svg_store.get(handle).unwrap();
                *mesh = tesselation::generate_mesh(&svg, &mut *fill_tess, &mut *stroke_tess);
            },
            AssetEvent::Removed { handle } => {
                let old_mesh = svg_mesh_map.0.remove(handle);

                if let Some(mesh_handle) = old_mesh {
                    mesh_store.remove(mesh_handle);
                }
            },
        }
    }
}

mod svg {
    use bevy::prelude::*;
    use lyon_geom::{Transform, Vector};
    use lyon_svg::parser::ViewBox;
    use lyon_tessellation::{self, math::Point};

    use crate::svg::{DrawType, PathDescriptor, Svg};

    pub fn parse_svg<'s>(svg_tree: usvg::Tree) -> Svg {
        let view_box = svg_tree.svg_node().view_box;
        let size = svg_tree.svg_node().size;
        let origin_center = Vector::new(-size.width() as f32 / 2., -size.height() as f32 / 2.);

        let mut descriptors = Vec::new();

        for node in svg_tree.root().descendants() {
            if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                // For some reason transform has sometimes negative scale values.
                // Here we correct to positive values.
                let (correct_scale_x, correct_scale_y) = (
                    if p.transform.a < 0.0 { -1.0 } else { 1.0 },
                    if p.transform.d < 0.0 { -1.0 } else { 1.0 },
                );
                let correct_scale = Transform::scale(correct_scale_x, correct_scale_y);

                let mut transform = Transform::new(
                    p.transform.a as f32,
                    p.transform.b as f32,
                    p.transform.c as f32,
                    p.transform.d as f32,
                    p.transform.e as f32,
                    p.transform.f as f32
                );
                transform = transform.pre_scale(correct_scale_x, correct_scale_y);
                transform = transform.then_translate(origin_center);

                if let Some(ref fill) = p.fill {
                    let color = match fill.paint {
                        usvg::Paint::Color(c) => {
                            Color::rgba_u8(c.red, c.green, c.blue, fill.opacity.to_u8())
                        },
                        _ => Color::default(),
                    };

                    descriptors.push(PathDescriptor {
                        segments: convert_path(p)
                            .map(|p| p.transformed(&correct_scale))
                            .collect(),
                        transform,
                        color,
                        draw_type: DrawType::Fill,
                    });
                }

                if let Some(ref stroke) = p.stroke {
                    let (color, stroke_opts) = convert_stroke(stroke);

                    descriptors.push(PathDescriptor {
                        segments: convert_path(p)
                            .map(|p| p.transformed(&correct_scale))
                            .collect(),
                        transform,
                        color,
                        draw_type: DrawType::Stroke(stroke_opts),
                    });
                }
            }
        }

        Svg {
            width:    size.width(),
            height:   size.height(),
            view_box: ViewBox {
                x: view_box.rect.x(),
                y: view_box.rect.y(),
                w: view_box.rect.width(),
                h: view_box.rect.height(),
            },
            paths:    descriptors,
        }
    }

    // Taken from https://github.com/nical/lyon/blob/74e6b137fea70d71d3b537babae22c6652f8843e/examples/wgpu_svg/src/main.rs
    struct PathConvIter<'a> {
        iter:      std::slice::Iter<'a, usvg::PathSegment>,
        prev:      Point,
        first:     Point,
        needs_end: bool,
        deferred:  Option<lyon_svg::path::PathEvent>,
    }

    impl<'l> Iterator for PathConvIter<'l> {
        type Item = lyon_svg::path::PathEvent;

        fn next(&mut self) -> Option<Self::Item> {
            use lyon_svg::path::PathEvent;
            if self.deferred.is_some() {
                return self.deferred.take();
            }

            let next = self.iter.next();
            match next {
                Some(usvg::PathSegment::MoveTo { x, y }) => {
                    if self.needs_end {
                        let last = self.prev;
                        let first = self.first;
                        self.needs_end = false;
                        self.prev = point(x, y);
                        self.deferred = Some(PathEvent::Begin { at: self.prev });
                        self.first = self.prev;
                        Some(PathEvent::End {
                            last,
                            first,
                            close: false,
                        })
                    } else {
                        self.first = point(x, y);
                        Some(PathEvent::Begin { at: self.first })
                    }
                },
                Some(usvg::PathSegment::LineTo { x, y }) => {
                    self.needs_end = true;
                    let from = self.prev;
                    self.prev = point(x, y);
                    Some(PathEvent::Line {
                        from,
                        to: self.prev,
                    })
                },
                Some(usvg::PathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                }) => {
                    self.needs_end = true;
                    let from = self.prev;
                    self.prev = point(x, y);
                    Some(PathEvent::Cubic {
                        from,
                        ctrl1: point(x1, y1),
                        ctrl2: point(x2, y2),
                        to: self.prev,
                    })
                },
                Some(usvg::PathSegment::ClosePath) => {
                    self.needs_end = false;
                    self.prev = self.first;
                    Some(PathEvent::End {
                        last:  self.prev,
                        first: self.first,
                        close: true,
                    })
                },
                None => {
                    if self.needs_end {
                        self.needs_end = false;
                        let last = self.prev;
                        let first = self.first;
                        Some(PathEvent::End {
                            last,
                            first,
                            close: false,
                        })
                    } else {
                        None
                    }
                },
            }
        }
    }

    fn point(x: &f64, y: &f64) -> Point {
        Point::new((*x) as f32, (*y) as f32)
    }

    fn convert_path<'a>(p: &'a usvg::Path) -> PathConvIter<'a> {
        PathConvIter {
            iter:      p.data.iter(),
            first:     Point::new(0.0, 0.0),
            prev:      Point::new(0.0, 0.0),
            deferred:  None,
            needs_end: false,
        }
    }

    fn convert_stroke(s: &usvg::Stroke) -> (Color, lyon_tessellation::StrokeOptions) {
        let color = match s.paint {
            usvg::Paint::Color(c) => Color::rgba_u8(c.red, c.green, c.blue, s.opacity.to_u8()),
            _ => Color::default(),
        };

        let linecap = match s.linecap {
            usvg::LineCap::Butt => lyon_tessellation::LineCap::Butt,
            usvg::LineCap::Square => lyon_tessellation::LineCap::Square,
            usvg::LineCap::Round => lyon_tessellation::LineCap::Round,
        };
        let linejoin = match s.linejoin {
            usvg::LineJoin::Miter => lyon_tessellation::LineJoin::Miter,
            usvg::LineJoin::Bevel => lyon_tessellation::LineJoin::Bevel,
            usvg::LineJoin::Round => lyon_tessellation::LineJoin::Round,
        };

        let opt = lyon_tessellation::StrokeOptions::tolerance(0.01)
            .with_line_width(s.width.value() as f32)
            .with_line_cap(linecap)
            .with_line_join(linejoin);

        (color, opt)
    }
}

mod tesselation {
    use bevy::log::error;
    use bevy::render::mesh::Mesh;
    use lyon_tessellation::{
        self, BuffersBuilder, FillOptions, FillTessellator, StrokeTessellator,
    };

    use crate::{
        svg::{DrawType, Svg},
        vertex_buffer::{VertexBuffers, VertexConstructor, to_mesh, apply_transform, merge_buffers},
    };

    pub fn generate_mesh(
        svg: &Svg,
        fill_tess: &mut FillTessellator,
        stroke_tess: &mut StrokeTessellator,
    ) -> Mesh {
        let mut vertex_buffers = Vec::new();

        // TODO: still need to do something about the color, it is pretty washed out
        let mut color = None;

        for path in svg.paths.iter() {
            let mut vertex_buffer = VertexBuffers::new();

            if color.is_none() {
                color = Some(path.color);
            }

            match path.draw_type {
                DrawType::Fill => {
                    if let Err(e) = fill_tess.tessellate(
                        path.segments.clone(),
                        &FillOptions::tolerance(0.001),
                        &mut BuffersBuilder::new(&mut vertex_buffer, VertexConstructor {
                            color: path.color,
                        }),
                    ) {
                        error!("FillTessellator error: {:?}", e)
                    }
                },
                DrawType::Stroke(opts) => {
                    if let Err(e) = stroke_tess.tessellate(
                        path.segments.clone(),
                        &opts,
                        &mut BuffersBuilder::new(&mut vertex_buffer, VertexConstructor {
                            color: path.color,
                        }),
                    ) {
                        error!("StrokeTessellator error: {:?}", e)
                    }
                },
            }

            apply_transform(&mut vertex_buffer, path.transform);
            vertex_buffers.push(vertex_buffer);
        }

        to_mesh(merge_buffers(vertex_buffers))
    }
}
