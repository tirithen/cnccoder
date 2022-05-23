use anyhow::{anyhow, Result};

use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct Plane {
    pub start: Vector3,
    pub size: Vector2,
    pub end_z: f64,
    pub end_z_stop: f64,
    pub max_step_z: f64,
}

impl Plane {
    #[must_use]
    pub fn new(start: Vector3, size: Vector2, end_z: f64, max_step_z: f64) -> Self {
        Self {
            start,
            size,
            end_z,
            end_z_stop: end_z,
            max_step_z,
        }
    }

    #[must_use]
    pub fn new_with_slope(
        start: Vector3,
        size: Vector2,
        end_z: f64,
        end_z_stop: f64,
        max_step_z: f64,
    ) -> Self {
        Self {
            start,
            size,
            end_z,
            end_z_stop,
            max_step_z,
        }
    }

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

    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        let tool_radius = context.tool().radius();
        let tool_diameter = context.tool().diameter();

        if self.size.x < tool_diameter {
            return Err(anyhow!("Unable to plane area, tool is {} mm to wider than x dimension (tool diameter is {} mm)", tool_diameter - self.size.x, tool_diameter));
        }

        if self.size.y < tool_diameter {
            return Err(anyhow!("Unable to plane area, tool is {} mm to wider than y dimension (tool diameter is {} mm)", tool_diameter - self.size.y, tool_diameter));
        }

        let mut instructions = Vec::new();

        instructions.append(&mut vec![
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment {
                text: format!(
                    "Do planing at: x = {}, y = {}, size = {}",
                    round_precision(self.start.x),
                    round_precision(self.start.y),
                    self.size
                ),
            }),
            Instruction::G0(G0 {
                x: None,
                y: None,
                z: Some(context.z_safe()),
            }),
            Instruction::G0(G0 {
                x: Some(self.start.x + tool_radius),
                y: Some(self.start.y + tool_radius),
                z: None,
            }),
            Instruction::G1(G1 {
                x: None,
                y: None,
                z: Some(self.start.z),
                f: Some(context.tool().feed_rate()),
            }),
        ]);

        let delta_z = self.end_z_stop - self.end_z;
        let max_step_z = self.max_step_z.abs();
        let layers = if (self.end_z - self.end_z_stop).abs() < 0.01 {
            ((self.end_z - self.start.z).abs() / max_step_z).ceil() as u32
        } else {
            (delta_z.abs() / max_step_z).ceil() as u32
        };
        let start_z = if delta_z < 0.0 {
            self.start.z - delta_z
        } else {
            self.start.z
        };
        let mut end_z = start_z;
        let mut end_z_stop = start_z + delta_z;

        for _layer in 1..layers {
            end_z -= max_step_z;
            end_z_stop -= max_step_z;
            instructions.append(&mut self.generate_layer_instructions(
                end_z.min(context.z_safe()),
                end_z_stop.min(context.z_safe()),
                tool_radius,
            ));
        }

        instructions.append(&mut self.generate_layer_instructions(
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
        end_z: f64,
        end_z_stop: f64,
        tool_radius: f64,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let size_y = self.size.y - tool_radius * 2.0;
        let passes = (size_y / (tool_radius * 1.8)).ceil() as i32;
        let pass_y = size_y / passes as f64;

        instructions.push(Instruction::G1(G1 {
            x: Some(self.start.x + self.size.x - tool_radius),
            y: None,
            z: Some(end_z_stop),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(self.start.y + self.size.y - tool_radius),
            z: None,
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: Some(self.start.x + tool_radius),
            y: None,
            z: Some(end_z),
            f: None,
        }));

        instructions.push(Instruction::G1(G1 {
            x: None,
            y: Some(self.start.y + tool_radius),
            z: None,
            f: None,
        }));

        let mut end_at_start = true;

        for index in 0..passes {
            instructions.push(Instruction::G1(G1 {
                x: None,
                y: Some(self.start.y + tool_radius + index as f64 * pass_y),
                z: None,
                f: None,
            }));

            if index % 2 == 0 {
                instructions.push(Instruction::G1(G1 {
                    x: Some(self.start.x + self.size.x - tool_radius),
                    y: None,
                    z: Some(end_z_stop),
                    f: None,
                }));

                end_at_start = false;
            } else {
                instructions.push(Instruction::G1(G1 {
                    x: Some(self.start.x + tool_radius),
                    y: None,
                    z: Some(end_z),
                    f: None,
                }));

                end_at_start = true;
            }
        }

        instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(if end_at_start { end_z } else { end_z_stop } + 0.5),
        }));

        instructions.push(Instruction::G0(G0 {
            x: Some(self.start.x + tool_radius),
            y: Some(self.start.y + tool_radius),
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
