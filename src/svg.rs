use bevy::{prelude::*, reflect::TypeUuid};
use lyon_svg::parser::ViewBox;
use lyon_tessellation::{self};
use lyon_geom::Transform;

/// A loaded and deserialized SVG file.
#[derive(Debug, TypeUuid)]
#[uuid = "a37184af-d432-4003-80b1-db0a5c3c1083"]
pub struct Svg {
    /// Width of the SVG.
    pub width:    f64,
    /// Height of the SVG.
    pub height:   f64,
    /// ViewBox of the SVG.
    pub view_box: ViewBox,
    /// Content of the SVG.
    pub paths:    Vec<PathDescriptor>,
}

#[derive(Debug)]
pub struct PathDescriptor {
    pub segments:  Vec<lyon_svg::path::PathEvent>,
    pub transform: Transform<f32>,
    pub color:     Color,
    pub draw_type: DrawType,
}

#[derive(Debug)]
pub enum DrawType {
    Fill,
    Stroke(lyon_tessellation::StrokeOptions),
}
