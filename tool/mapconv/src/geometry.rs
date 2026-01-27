use crate::coord::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Point,
    Rect { size: DVec2 },
    Ellipse { size: DVec2 },
    Polygon { points: Vec<DVec2> },
    Polyline { points: Vec<DVec2> },
    Unshape,
}

impl Shape {
    pub fn has_area(&self) -> bool {
        match self {
            Shape::Point => false,
            Shape::Rect { size } => size.is_finite() && size.x.round() * size.y.round() >= 1.0,
            Shape::Ellipse { .. } => false,
            Shape::Polygon { .. } => false,
            Shape::Polyline { .. } => false,
            Shape::Unshape => false,
        }
    }
}

impl From<&tiled::ObjectShape> for Shape {
    fn from(value: &tiled::ObjectShape) -> Self {
        match value {
            tiled::ObjectShape::Point(..) => Self::Point,
            tiled::ObjectShape::Rect { width, height } => Self::Rect {
                size: DVec2::new(*width as f64, *height as f64).abs(),
            },
            tiled::ObjectShape::Ellipse { width, height } => Self::Ellipse {
                size: DVec2::new(*width as f64, *height as f64).abs(),
            },
            tiled::ObjectShape::Polyline { points } => Self::Polyline {
                points: points
                    .iter()
                    .map(|(x, y)| DVec2::new(*x as f64, *y as f64))
                    .collect(),
            },
            tiled::ObjectShape::Polygon { points } => Self::Polygon {
                points: points
                    .iter()
                    .map(|(x, y)| DVec2::new(*x as f64, *y as f64))
                    .collect(),
            },
            tiled::ObjectShape::Text { .. } => Self::Unshape,
        }
    }
}
