use anyhow::Result;

use crate::instructions::*;
use crate::program::*;
use crate::types::*;

mod arc;
pub use arc::*;

mod circle;
pub use circle::*;

mod frame;
pub use frame::*;

mod line;
pub use line::*;

mod path;
pub use path::*;

mod area;
pub use area::*;

#[derive(Debug, Clone)]
pub enum Cut {
    Arc(Arc),
    Circle(Circle),
    Frame(Frame),
    Line(Line),
    Path(Path),
    Area(Area),
}

impl Cut {
    #[must_use]
    pub fn circle(start: Vector3, end_z: f64, radius: f64, max_step_z: f64) -> Self {
        Self::Circle(Circle::new(
            start,
            end_z,
            radius,
            max_step_z,
            ToolPathCompensation::None,
        ))
    }

    #[must_use]
    pub fn circle_inner(start: Vector3, end_z: f64, radius: f64, max_step_z: f64) -> Self {
        Self::Circle(Circle::new(
            start,
            end_z,
            radius,
            max_step_z,
            ToolPathCompensation::Inner,
        ))
    }

    #[must_use]
    pub fn circle_outer(start: Vector3, end_z: f64, radius: f64, max_step_z: f64) -> Self {
        Self::Circle(Circle::new(
            start,
            end_z,
            radius,
            max_step_z,
            ToolPathCompensation::Outer,
        ))
    }

    #[must_use]
    pub fn drill(start: Vector3, end_z: f64) -> Self {
        Self::Circle(Circle::drill(start, end_z))
    }

    #[must_use]
    pub fn arc(
        from: Vector3,
        to: Vector3,
        center: Vector3,
        axis: Axis,
        direction: Direction,
    ) -> Self {
        Self::Arc(Arc::new(from, to, center, axis, direction))
    }

    #[must_use]
    pub fn line(from: Vector3, to: Vector3) -> Self {
        Self::Line(Line::new(from, to))
    }

    #[must_use]
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }

    #[must_use]
    pub fn frame(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Frame(Frame::new(
            start,
            size,
            end_z,
            max_step_z,
            ToolPathCompensation::None,
        ))
    }

    #[must_use]
    pub fn frame_inner(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Frame(Frame::new(
            start,
            size,
            end_z,
            max_step_z,
            ToolPathCompensation::Inner,
        ))
    }

    #[must_use]
    pub fn frame_outer(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Frame(Frame::new(
            start,
            size,
            end_z,
            max_step_z,
            ToolPathCompensation::Outer,
        ))
    }

    #[must_use]
    pub fn plane(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Area(Area::new(
            start,
            size,
            end_z,
            max_step_z,
            ToolPathCompensation::Outer,
        ))
    }

    #[must_use]
    pub fn pocket(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Area(Area::new(
            start,
            size,
            end_z,
            max_step_z,
            ToolPathCompensation::Inner,
        ))
    }

    #[must_use]
    pub fn plane_with_slope(
        start: Vector3,
        size: Vector2,
        end_z: f64,
        end_z_stop: f64,
        max_step_z: f64,
    ) -> Self {
        Self::Area(Area::new_with_slope(
            start,
            size,
            end_z,
            end_z_stop,
            max_step_z,
            ToolPathCompensation::Outer,
        ))
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Arc(c) => c.bounds(),
            Self::Circle(c) => c.bounds(),
            Self::Frame(c) => c.bounds(),
            Self::Line(c) => c.bounds(),
            Self::Path(c) => c.bounds(),
            Self::Area(c) => c.bounds(),
        }
    }

    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        match self {
            Self::Arc(c) => c.to_instructions(context),
            Self::Circle(c) => c.to_instructions(context),
            Self::Frame(c) => c.to_instructions(context),
            Self::Line(c) => c.to_instructions(context),
            Self::Path(c) => c.to_instructions(context),
            Self::Area(c) => c.to_instructions(context),
        }
    }
}
