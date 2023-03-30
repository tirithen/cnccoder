use anyhow::{anyhow, Result};

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

/// Cut a frame around an area. Unlike [Area](struct.Area.html), the frame cut will only cut at the edge
/// of the area, and not cut inside.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Start point in 3D space.
    pub start: Vector3,
    /// Size of the area.
    pub size: Vector2,
    /// The end depth of the cut on the z axis.
    pub end_z: f64,
    /// The maximum depth to cut on the z axis on each pass.
    pub max_step_z: f64,
    /// Indicates how a path should be compensated by the radius of the tool.
    /// `ToolPathCompensation::Inner` is useful for cutting rectangular holes,
    /// `ToolPathCompensation::Outer` is useful for cutting out rectangle
    /// pieces.
    pub compensation: ToolPathCompensation,
}

impl Frame {
    /// Creates a new `Frame` struct.
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
            max_step_z,
            compensation,
        }
    }

    /// Returns the bounds of the cut.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector3::new(self.start.x, self.start.y, self.end_z),
            max: Vector3::new(
                self.start.x + self.size.x,
                self.start.y + self.size.y,
                self.start.z,
            ),
        }
    }

    /// Converts the struct to G-code instructions.
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        let tool_radius = context.tool().radius();
        let tool_diameter = context.tool().diameter();
        let tool_units = context.tool().units();

        if self.size.x < tool_diameter {
            // TODO: handle calculation for the case when tool and program units are different.
            return Err(anyhow!("Unable to cut frame, tool is {:.2} {} to wider than x dimension (tool diameter is {:.2} {})", tool_diameter - self.size.x, tool_units, tool_diameter, tool_units));
        }

        if self.size.y < tool_diameter {
            // TODO: handle calculation for the case when tool and program units are different.
            return Err(anyhow!("Unable to cut frame, tool is {:.2} {} to wider than y dimension (tool diameter is {:.2} {})", tool_diameter - self.size.y, tool_units, tool_diameter, tool_units));
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
                    "Cut frame: x = {}, y = {}, size = {}",
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

        let max_step_z = self.max_step_z.abs();
        let mut start_z = start.z;
        let mut end_z = start_z;
        let layers = ((start_z - self.end_z).abs() / max_step_z).floor() as u32;

        for _layer in 1..=layers {
            end_z -= max_step_z;
            instructions.append(&mut self.generate_layer_instructions(start, size, start_z, end_z));
            start_z = end_z;
        }

        instructions
            .append(&mut self.generate_layer_instructions(start, size, self.end_z, self.end_z));

        instructions.push(Instruction::G1(G1 {
            x: Some(start.x + size.x),
            y: None,
            z: None,
            f: None,
        }));

        instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(context.z_safe()),
        }));

        instructions.push(Instruction::G0(G0 {
            x: Some(start.x),
            y: Some(start.y),
            z: None,
        }));

        Ok(instructions)
    }

    fn generate_layer_instructions(
        &self,
        start: Vector3,
        size: Vector2,
        start_z: f64,
        end_z: f64,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let size_x = size.x;
        let size_y = size.y;
        let delta_z = end_z - start_z;
        let circumference = (size_x + size_y) * 2.0;
        let x_step_z = (size_x / circumference) * delta_z;
        let y_step_z = (size_y / circumference) * delta_z;

        instructions.push(Instruction::G1(G1 {
            x: Some(start.x + size.x),
            y: None,
            z: Some(start_z + x_step_z),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(start.y + size.y),
            z: Some(start_z + x_step_z + y_step_z),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: Some(start.x),
            y: None,
            z: Some(start_z + x_step_z * 2.0 + y_step_z),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(start.y),
            z: Some(end_z),
            f: None,
        }));

        instructions
    }
}
