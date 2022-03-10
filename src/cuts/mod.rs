use crate::instructions::*;
use crate::program::*;
use crate::types::*;

mod circle;
pub use circle::*;

mod path;
pub use path::*;

mod plane;
pub use plane::*;

#[derive(Debug, Clone)]
pub enum Cut {
    Circle(Circle),
    Path(Path),
    Plane(Plane),
}

impl Cut {
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Circle(c) => c.bounds(),
            Self::Path(c) => c.bounds(),
            Self::Plane(c) => c.bounds(),
        }
    }

    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        match self {
            Self::Circle(cut) => cut.to_instructions(context),
            Self::Path(cut) => cut.to_instructions(context),
            Self::Plane(cut) => cut.to_instructions(context),
        }
    }
}
