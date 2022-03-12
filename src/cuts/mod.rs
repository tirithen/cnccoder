use crate::instructions::*;
use crate::program::*;
use crate::types::*;

mod circle;
pub use circle::*;

mod frame;
pub use frame::*;

mod path;
pub use path::*;

mod plane;
pub use plane::*;

#[derive(Debug, Clone)]
pub enum Cut {
    Circle(Circle),
    Frame(Frame),
    Path(Path),
    Plane(Plane),
}

impl Cut {
    #[must_use]
    pub fn circle(start: Vector3, end_z: f64, radius: f64, max_step_z: f64) -> Self {
        Self::Circle(Circle::new(start, end_z, radius, max_step_z))
    }

    #[must_use]
    pub fn hole(start: Vector3, end_z: f64) -> Self {
        Self::Circle(Circle::hole(start, end_z))
    }

    #[must_use]
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }

    #[must_use]
    pub fn frame(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Frame(Frame::new(start, size, end_z, max_step_z))
    }

    #[must_use]
    pub fn plane(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self::Plane(Plane::new(start, size, end_z, max_step_z))
    }

    #[must_use]
    pub fn plane_with_slope(start: Vector3, size: Vector2, end_z: f64, end_z_stop: f64, max_step_z: f64) -> Self {
        Self::Plane(Plane::new_with_slope(start, size, end_z, end_z_stop, max_step_z))
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Circle(c) => c.bounds(),
            Self::Frame(c) => c.bounds(),
            Self::Path(c) => c.bounds(),
            Self::Plane(c) => c.bounds(),
        }
    }

    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        match self {
            Self::Circle(c) => c.to_instructions(context),
            Self::Frame(c) => c.to_instructions(context),
            Self::Path(c) => c.to_instructions(context),
            Self::Plane(c) => c.to_instructions(context),
        }
    }
}
