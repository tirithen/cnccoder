use anyhow::{anyhow, Result};

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

/// Cut a circle spiraling down in a helix to `end_z`.
///
/// If the circle radius equals the tool radius with `ToolPathCompensation::None` the cut will
/// instead be a drilling top/down cut. Unlike [Area](struct.Area.html), the circle cut will
/// only cut at the edge of the circle, and not cut inside the circle.
#[derive(Debug, Clone)]
pub struct Circle {
    /// Start point in 3D space.
    pub start: Vector3,
    /// The circle radius
    pub radius: f64,
    /// The end depth of the cut on the z axis.
    pub end_z: f64,
    /// The maximum depth to cut on the z axis on each pass.
    pub max_step_z: f64,
    /// Indicates how a path should be compensated by the radius of the tool.
    /// `ToolPathCompensation::Inner` is useful for cutting holes wider than the tool,
    /// `ToolPathCompensation::Outer` is useful for cutting out round pieces, and
    /// `ToolPathCompensation::Outer` is useful when drilling.
    pub compensation: ToolPathCompensation,
}

impl Circle {
    /// Creates a new `Circle` struct.
    #[must_use]
    pub fn new(
        start: Vector3,
        radius: f64,
        end_z: f64,
        max_step_z: f64,
        compensation: ToolPathCompensation,
    ) -> Self {
        Self {
            start,
            end_z,
            radius,
            max_step_z,
            compensation,
        }
    }

    /// Drill cut from start coordinate to end z depth.
    #[must_use]
    pub fn drill(start: Vector3, end_z: f64) -> Self {
        Self {
            start,
            radius: 0.0,
            end_z,
            max_step_z: 0.0,
            compensation: ToolPathCompensation::None,
        }
    }

    /// Returns the bounds of the cut.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector3::new(
                self.start.x - self.radius,
                self.start.y - self.radius,
                self.end_z,
            ),
            max: Vector3::new(
                self.start.x + self.radius,
                self.start.y + self.radius,
                self.start.z,
            ),
        }
    }

    /// Converts the struct to G-code instructions.
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        let mut instructions = vec![];

        let tool_radius = context.tool().radius();
        let cut_radius = match self.compensation {
            ToolPathCompensation::None => self.radius,
            ToolPathCompensation::Inner => self.radius - tool_radius,
            ToolPathCompensation::Outer => self.radius + tool_radius,
        };

        if (0.0..0.001).contains(&cut_radius) {
            instructions.append(&mut vec![
                Instruction::Empty(Empty {}),
                Instruction::Comment(Comment {
                    text: format!(
                        "Drill hole at: x = {}, y = {}",
                        round_precision(self.start.x),
                        round_precision(self.start.y)
                    ),
                }),
                Instruction::G0(G0 {
                    x: None,
                    y: None,
                    z: Some(context.z_safe()),
                }),
                Instruction::G0(G0 {
                    x: Some(self.start.x),
                    y: Some(self.start.y),
                    z: None,
                }),
                Instruction::G1(G1 {
                    x: None,
                    y: None,
                    z: Some(self.end_z),
                    f: Some(context.tool().feed_rate()),
                }),
                Instruction::G0(G0 {
                    x: None,
                    y: None,
                    z: Some(context.z_safe()),
                }),
            ])
        } else if cut_radius > 0.0 {
            instructions.append(&mut vec![
                Instruction::Empty(Empty {}),
                Instruction::Comment(Comment {
                    text: format!(
                        "Cut hole at: x = {}, y = {}",
                        round_precision(self.start.x),
                        round_precision(self.start.y)
                    ),
                }),
                Instruction::G0(G0 {
                    x: None,
                    y: None,
                    z: Some(context.z_safe()),
                }),
                Instruction::G0(G0 {
                    x: Some(self.start.x - cut_radius),
                    y: Some(self.start.y),
                    z: None,
                }),
                Instruction::G1(G1 {
                    x: None,
                    y: None,
                    z: Some(self.start.z),
                    f: Some(context.tool().feed_rate()),
                }),
            ]);

            let max_step_z = self.max_step_z.abs();

            // TODO: add check that layer steps does not exceed cutting height if the bit
            let layers = ((self.start.z - self.end_z) / max_step_z).floor() as u32;

            // Cut spiraling down in steps
            for index in 0..layers {
                instructions.push(Instruction::G2(G2 {
                    x: Some(self.start.x - cut_radius),
                    y: None,
                    z: Some((self.start.z - index as f64 * max_step_z).max(self.end_z)),
                    i: Some(cut_radius),
                    j: None,
                    k: None,
                    r: None,
                    p: None,
                    f: None,
                }));
            }

            // Extra flat circle
            instructions.push(Instruction::G2(G2 {
                x: Some(self.start.x - cut_radius),
                y: None,
                z: Some(self.end_z),
                i: Some(cut_radius),
                j: None,
                k: None,
                r: None,
                p: None,
                f: None,
            }));

            instructions.push(Instruction::G2(G2 {
                x: Some(self.start.x - cut_radius),
                y: None,
                z: Some(self.end_z),
                i: Some(cut_radius - 0.001),
                j: None,
                k: None,
                r: None,
                p: None,
                f: None,
            }));

            instructions.push(Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }));
        } else {
            let tool = context.tool();
            let units = context.units();

            // TODO: handle calculation for the case when tool and program units are different.
            return Err(anyhow!(
                "Unable to cut circle of diameter {:.2} {} with tool diameter {:.2} {}.",
                cut_radius.abs() * 2.0,
                units,
                tool.diameter(),
                units,
            ));
        }

        Ok(instructions)
    }
}
