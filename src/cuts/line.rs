use anyhow::Result;

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

/// Linear move from one 3D point to another.
#[derive(Debug, Clone)]
pub struct Line {
    /// Starting point in 3D space.
    pub from: Vector3,
    /// End point in 3D space.
    pub to: Vector3,
}

impl Line {
    /// Creates an `Line` struct.
    #[must_use]
    pub fn new(from: Vector3, to: Vector3) -> Self {
        Self { from, to }
    }

    /// Bounds in 3D space for the linear move.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        let max_x = self.from.x.max(self.to.x);
        let min_x = self.from.x.min(self.to.x);
        let max_y = self.from.y.max(self.to.y);
        let min_y = self.from.y.min(self.to.y);
        let max_z = self.from.z.max(self.to.z);
        let min_z = self.from.z.min(self.to.z);

        Bounds {
            min: Vector3::new(min_x, min_y, min_z),
            max: Vector3::new(max_x, max_y, max_z),
        }
    }

    /// Converts the struct to G-code instructions.
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        let mut instructions = vec![];

        instructions.append(&mut vec![
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment {
                text: format!(
                    "Cut line from: x = {}, y = {}, z = {}, to:  x = {}, y = {}, z = {}",
                    round_precision(self.from.x),
                    round_precision(self.from.y),
                    round_precision(self.from.z),
                    round_precision(self.to.x),
                    round_precision(self.to.y),
                    round_precision(self.to.z),
                ),
            }),
            Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }),
            Instruction::G0(G0 {
                x: Some(self.from.x),
                y: Some(self.from.y),
                z: None,
            }),
            Instruction::G1(G1 {
                x: None,
                y: None,
                z: Some(self.from.z),
                f: Some(context.tool().feed_rate()),
            }),
            Instruction::G1(G1 {
                x: Some(self.to.x),
                y: Some(self.to.y),
                z: Some(self.to.z),
                f: None,
            }),
            Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }),
        ]);

        Ok(instructions)
    }
}
