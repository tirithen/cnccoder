use crate::instructions::*;
use crate::program::*;
use crate::types::*;

mod path;
pub use path::*;

#[derive(Debug, Clone)]
pub enum Cut {
    Path(Path),
}

impl Cut {
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Path(c) => c.bounds(),
        }
    }

    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        match self {
            Self::Path(cut) => cut.to_instructions(context),
        }
    }
}
