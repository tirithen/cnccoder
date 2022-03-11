use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct Circle {
    pub start: Vector3,
    pub end_z: f64,
    pub radius: f64,
    pub max_step_z: f64,
}

impl Circle {
    #[must_use]
    pub fn new(start: Vector3, end_z: f64, radius: f64, max_step_z: f64) -> Self {
        Self {
            start,
            end_z,
            radius,
            max_step_z,
        }
    }

    #[must_use]
    pub fn hole(start: Vector3, end_z: f64) -> Self {
        Self {
            start,
            end_z,
            radius: 0.0,
            max_step_z: 0.0,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        Bounds {
            min: Vector3::new(self.start.x - self.radius, self.start.y - self.radius, self.end_z),
            max: Vector3::new(self.start.x + self.radius, self.start.y + self.radius, self.start.z),
        }
    }

    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        let mut instructions = vec![];

        if self.radius < 0.001 {
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
        } else if self.radius >= 0.0 {
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
                    x: Some(self.start.x - self.radius),
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
                    x: Some(self.start.x - self.radius),
                    y: None,
                    z: Some((self.start.z - index as f64 * max_step_z).max(self.end_z)),
                    i: Some(self.radius),
                    j: None,
                    k: None,
                    r: None,
                    p: None,
                    f: None,
                }));
            }

            // Extra flat circle
            instructions.push(Instruction::G2(G2 {
                x: Some(self.start.x - self.radius),
                y: None,
                z: Some(self.end_z),
                i: Some(self.radius),
                j: None,
                k: None,
                r: None,
                p: None,
                f: None,
            }));

            instructions.push(Instruction::G2(G2 {
                x: Some(self.start.x - self.radius),
                y: None,
                z: Some(self.end_z),
                i: Some(self.radius - 0.001),
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
            panic!("Unable to make circle, tool is to wide");
        }

        instructions
    }
}
