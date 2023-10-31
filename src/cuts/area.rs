use anyhow::{anyhow, Result};

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

/// Surface cut an area, can be used for both planing and rectangular pockets.
#[derive(Debug, Clone)]
pub struct Area {
    /// Start point in 3D space.
    pub start: Vector3,
    /// Size of the area.
    pub size: Vector2,
    /// The end depth of the cut on the z axis.
    pub end_z: f64,
    /// The end depth to stop at on the slope (used for deprecated
    /// [Area::new_with_slope](struct.Area.html#method.new_with_slope) method).
    /// For future compatability it can be set to the same value as `end_z`, or
    /// use [Area::new](struct.Area.html#method.new) method that does this
    /// internally already.
    #[deprecated(
        since = "0.1.0",
        note = "Only works in one direction, will likely be removed in future releases."
    )]
    pub end_z_stop: f64,
    /// The maximum depth to cut on the z axis on each pass.
    pub max_step_z: f64,
    /// Indicates how a path should be compensated by the radius of the tool.
    /// `ToolPathCompensation::Inner` is useful for pocket cuts,
    /// `ToolPathCompensation::Outer` is useful for cutting out rectangle
    /// pieces.
    pub compensation: ToolPathCompensation,
}

#[allow(deprecated)]
impl Area {
    /// Creates a new `Area` struct.
    #[must_use]
    pub fn new(
        start: Vector3,
        size: Vector2,
        end_z: f64,
        max_step_z: f64,
        compensation: ToolPathCompensation,
    ) -> Self {
        Self {
            start,
            size,
            end_z,
            end_z_stop: end_z,
            max_step_z,
            compensation,
        }
    }

    /// Creates a new `Area` struct that cuts a slope. This method is
    /// deprecated as it has no options to choose in which direction
    /// the slope should be cut, and will therefore likely be removed
    /// eventually.
    #[deprecated(
        since = "0.1.0",
        note = "Only works in one direction, will likely be removed in future releases."
    )]
    #[must_use]
    pub fn new_with_slope(
        start: Vector3,
        size: Vector2,
        end_z: f64,
        end_z_stop: f64,
        max_step_z: f64,
        compensation: ToolPathCompensation,
    ) -> Self {
        Self {
            start,
            size,
            end_z,
            end_z_stop,
            max_step_z,
            compensation,
        }
    }

    /// Returns the bounds of the cut.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector3::new(self.start.x, self.start.y, self.end_z.min(self.end_z_stop)),
            max: Vector3::new(
                self.start.x + self.size.x,
                self.start.y + self.size.y,
                self.start.z,
            ),
        }
    }

    /// Converts the struct to G-code instructions.
    pub fn to_instructions(&self, context: InnerContext) -> Result<Vec<Instruction>> {
        let tool_radius = context.tool().radius();
        let tool_diameter = context.tool().diameter();
        let tool_units = context.tool().units();

        if self.size.x < tool_diameter {
            // TODO: handle calculation for the case when tool and program units are different.
            return Err(anyhow!("Unable to plane area, tool is {:.2} {} wider than x dimension (tool diameter is {:.2} {})", tool_diameter - self.size.x, tool_units, tool_diameter, tool_units));
        }

        if self.size.y < tool_diameter {
            // TODO: handle calculation for the case when tool and program units are different.
            return Err(anyhow!("Unable to plane area, tool is {:.2} {} wider than y dimension (tool diameter is {:.2} {})", tool_diameter - self.size.y, tool_units, tool_diameter, tool_units));
        }

        let start = match self.compensation {
            ToolPathCompensation::None => self.start,
            ToolPathCompensation::Inner => self.start.add_x(tool_radius).add_y(tool_radius),
            ToolPathCompensation::Outer => self.start.add_x(-tool_radius).add_y(-tool_radius),
        };

        let size = match self.compensation {
            ToolPathCompensation::None => self.size,
            ToolPathCompensation::Inner => self
                .size
                .add_x(-tool_radius * 2.0)
                .add_y(-tool_radius * 2.0),
            ToolPathCompensation::Outer => {
                self.size.add_x(tool_radius * 2.0).add_y(tool_radius * 2.0)
            }
        };

        let mut instructions = Vec::new();

        instructions.append(&mut vec![
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment {
                text: format!(
                    "Do planing at: x = {}, y = {}, size = {}",
                    round_precision(start.x),
                    round_precision(start.y),
                    size
                ),
            }),
            Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }),
            Instruction::G0(G0 {
                x: Some(start.x),
                y: Some(start.y),
                z: None,
            }),
            Instruction::G1(G1 {
                x: None,
                y: None,
                z: Some(start.z),
                f: Some(context.tool().feed_rate()),
            }),
        ]);

        let delta_z = self.end_z_stop - self.end_z;
        let max_step_z = self.max_step_z.abs();
        let layers = if (self.end_z - self.end_z_stop).abs() < 0.01 {
            ((self.end_z - start.z).abs() / max_step_z).ceil() as u32
        } else {
            (delta_z.abs() / max_step_z).ceil() as u32
        };
        let start_z = if delta_z < 0.0 {
            start.z - delta_z
        } else {
            start.z
        };
        let mut end_z = start_z;
        let mut end_z_stop = start_z + delta_z;

        for _layer in 1..layers {
            end_z -= max_step_z;
            end_z_stop -= max_step_z;
            instructions.append(&mut self.generate_layer_instructions(
                start,
                size,
                end_z.min(context.z_safe()),
                end_z_stop.min(context.z_safe()),
                tool_radius,
            ));
        }

        instructions.append(&mut self.generate_layer_instructions(
            start,
            size,
            self.end_z.min(context.z_safe()),
            self.end_z_stop.min(context.z_safe()),
            tool_radius,
        ));

        instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(context.z_safe()),
        }));

        Ok(instructions)
    }

    fn generate_layer_instructions(
        &self,
        start: Vector3,
        size: Vector2,
        end_z: f64,
        end_z_stop: f64,
        tool_radius: f64,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let size_y = size.y;
        let passes = (size_y / (tool_radius * 1.8)).ceil() as i32;
        let pass_y = size_y / passes as f64;

        instructions.push(Instruction::G1(G1 {
            x: Some(start.x + size.x),
            y: None,
            z: Some(end_z_stop),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(start.y + size.y),
            z: None,
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: Some(start.x),
            y: None,
            z: Some(end_z),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(start.y),
            z: None,
            f: None,
        }));

        let mut end_at_start = true;

        if size.x > tool_radius * 2.0 {
            for index in 0..passes {
                instructions.push(Instruction::G1(G1 {
                    x: None,
                    y: Some(start.y + index as f64 * pass_y),
                    z: None,
                    f: None,
                }));

                if index % 2 == 0 {
                    instructions.push(Instruction::G1(G1 {
                        x: Some(start.x + size.x),
                        y: None,
                        z: Some(end_z_stop),
                        f: None,
                    }));

                    end_at_start = false;
                } else {
                    instructions.push(Instruction::G1(G1 {
                        x: Some(start.x),
                        y: None,
                        z: Some(end_z),
                        f: None,
                    }));

                    end_at_start = true;
                }
            }
        }

        instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(if end_at_start { end_z } else { end_z_stop } + 0.5),
        }));

        instructions.push(Instruction::G0(G0 {
            x: Some(start.x),
            y: Some(start.y),
            z: Some(end_z + 0.5),
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: None,
            z: Some(end_z),
            f: None,
        }));

        instructions
    }
}
