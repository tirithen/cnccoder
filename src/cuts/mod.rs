//! Module providing a variety of cuts that can be added to a program tool context.

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

/// Enum variant providing the cuts available for adding to a program.
#[derive(Debug, Clone)]
pub enum Cut {
    /// 3D arc where the axis to turn around can be selected.
    Arc(Arc),
    /// Top/down circle cut, that can also be used for drilling.
    Circle(Circle),
    /// Top/down rectangle frame/contour cut.
    Frame(Frame),
    /// 3D line cut between two points.
    Line(Line),
    /// Top/down path cut that is built from segments of various types.
    Path(Path),
    /// Top/down rectangle area cut that is useful for pocket cuts as
    /// well as for planing cuts.
    Area(Area),
}

impl Cut {
    /// Helper for creating top/down circle cuts without tool compensation.
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

    /// Helper for creating top/down circle cuts with inner tool compensation,
    /// for example useful when cutting holes.
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

    /// Helper for creating top/down circle cuts with outer tool compensation,
    /// for example useful when cutting out circle shapes.
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

    /// Helper for top/down drilling.
    #[must_use]
    pub fn drill(start: Vector3, end_z: f64) -> Self {
        Self::Circle(Circle::drill(start, end_z))
    }

    /// Helper for creating 3D arc cuts.
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

    /// Helper for creating 3D line cuts.
    #[must_use]
    pub fn line(from: Vector3, to: Vector3) -> Self {
        Self::Line(Line::new(from, to))
    }

    /// Helper for creating top/down path cuts consisting of several
    /// [Segment](enum.Segment.html) structs (lines, arcs, points).
    #[must_use]
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }

    /// Helper for creating top/down rectangle frame cuts without tool compensation
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

    /// Helper for creating top/down rectangle frame cuts with inner tool compensation,
    /// for example useful when cutting rectangle holes.
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

    /// Helper for creating top/down rectangle frame cuts with outer tool compensation,
    /// for example useful when cutting out rectangle shapes.
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

    /// Helper for creating top/down planing cuts.
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

    /// Helper for creating top/down pocket cuts.
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

    /// Helper for planing with a slope, deprecated so not recommended to use.
    #[deprecated(
        since = "0.1.0",
        note = "Only works in one direction, will likely be removed in future releases."
    )]
    #[allow(deprecated)]
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

    /// Calculates the bounds of the program.
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

    /// Converts the cuts to a list of G-code instructions
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
