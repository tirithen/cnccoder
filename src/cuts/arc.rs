use anyhow::{anyhow, Result};

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

/// Arc move from one 3D point to another on either of the xyz axis and in either rotation
/// direction.
///
/// It can be used to cut in a arc/circle or helix. It will be converted to
/// G2 and G3 G-code instructions.
#[derive(Debug, Clone)]
pub struct Arc {
    /// Starting point in 3D space.
    pub from: Vector3,
    /// End point in 3D space.
    pub to: Vector3,
    /// Center point in 3D space, will be used along with from and to to derive the radius.
    /// The center must be places so that it has the same distance between center -> from,
    /// and center -> to.
    pub center: Vector3,
    /// The axis to cut around, when `Axis::Z` is used the cut will be top/down.
    pub axis: Axis,
    /// The direction to cut the arc with.
    pub direction: Direction,
}

impl Arc {
    /// Creates an `Arc` struct.
    #[must_use]
    pub fn new(
        from: Vector3,
        to: Vector3,
        center: Vector3,
        axis: Axis,
        direction: Direction,
    ) -> Self {
        Self {
            from,
            to,
            center,
            axis,
            direction,
        }
    }

    /// Returns the radius of the arc.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.from
            .distance_to(self.center)
            .max(self.to.distance_to(self.center))
    }

    /// Bounds in 3D space for the arc move, currently this is not yet properly calculated.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector3::default(),
            max: Vector3::default(),
        }

        /*
        // TODO: implement a more proper bounds calculation for arc cuts
        let radius = self.radius();
        let max_x = self.center.x + radius;
        let min_x = self.center.x - radius;
        let max_y = self.center.y + radius;
        let min_y = self.center.y - radius;
        let max_z = self.center.y + radius;
        let min_z = self.center.y - radius;

        Bounds {
            min: Vector3::new(min_x, min_y, min_z),
            max: Vector3::new(max_x, max_y, max_z),
        }
        */
    }

    /// Converts arc to G-code instructions, will return error if the distance between
    /// center -> from does not equal center -> to.
    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        let distance_from = self.from.distance_to(self.center);
        let distance_to = self.to.distance_to(self.center);

        if (distance_from - distance_to).abs() > 0.0001 {
            return Err(anyhow!(
                "Arc distances from/center ({} {}) and to/center ({} {}) must be equal",
                distance_from,
                context.units(),
                distance_to,
                context.units(),
            ));
        }

        let mut instructions = vec![];

        instructions.append(&mut vec![
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment {
                text: format!(
                    "Cut arc {} at axis {}, from: x = {}, y = {}, z = {}, to:  x = {}, y = {}, z = {}",
                    self.direction,
                    self.axis,
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
        ]);

        match self.axis {
            Axis::X => {
                instructions.push(Instruction::G19(G19 {}));
            }
            Axis::Y => {
                instructions.push(Instruction::G18(G18 {}));
            }
            Axis::Z => {
                instructions.push(Instruction::G17(G17 {}));
            }
        }

        match self.direction {
            Direction::Clockwise => {
                instructions.push(Instruction::G2(G2 {
                    x: Some(self.to.x),
                    y: Some(self.to.y),
                    z: Some(self.to.z),
                    i: Some(self.center.x - self.from.x),
                    j: Some(self.center.y - self.from.y),
                    k: Some(self.center.z - self.from.z),
                    r: None,
                    p: None,
                    f: Some(context.tool().feed_rate()),
                }));
            }
            Direction::Counterclockwise => {
                instructions.push(Instruction::G3(G3 {
                    x: Some(self.to.x),
                    y: Some(self.to.y),
                    z: Some(self.to.z),
                    i: Some(self.center.x - self.from.x),
                    j: Some(self.center.y - self.from.y),
                    k: Some(self.center.z - self.from.z),
                    r: None,
                    p: None,
                    f: Some(context.tool().feed_rate()),
                }));
            }
        }

        instructions.append(&mut vec![
            Instruction::G17(G17 {}),
            Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }),
        ]);

        Ok(instructions)
    }
}
