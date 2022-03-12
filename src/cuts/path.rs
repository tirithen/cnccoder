use crate::instructions::*;
use crate::program::*;
use crate::types::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct Line {
    from: Vector2,
    to: Vector2,
}

impl Line {
    #[must_use]
    pub fn new(from: Vector2, to: Vector2) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone)]
pub struct Arc {
    pub from: Vector2,
    pub to: Vector2,
    pub center: Vector2,
}

impl Arc {
    #[must_use]
    pub fn new(from: Vector2, to: Vector2, center: Vector2) -> Self {
        Self { from, to, center }
    }
}

#[derive(Debug, Clone)]
pub enum Segment {
    Line(Line),
    Arc(Arc),
}

impl Segment {
    #[must_use]
    pub fn line(from: Vector2, to: Vector2) -> Self {
        Self::Line(Line::new(from, to))
    }

    #[must_use]
    pub fn arc(from: Vector2, to: Vector2, center: Vector2) -> Self {
        Self::Arc(Arc::new(from, to, center))
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    start: Vector3,
    segments: Vec<Segment>,
    end_z: f64,
    max_step_z: f64,
}

impl Path {
    #[must_use]
    pub fn new(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self {
            start,
            segments,
            end_z,
            max_step_z,
        }
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::minmax();

        for segment in self.segments.iter() {
            match segment {
                Segment::Arc(_arc) => println!("WARNING: bounding box is not calculated for Arc segments yet, ensure that dependent code has other ways of calculating relevant bounding box"),
                Segment::Line(line) => {
                    let max_x = self.start.x + if line.from.x > line.to.x {line.from.x} else {line.to.x};
                    if bounds.max.x < max_x {
                        bounds.max.x = max_x;
                    }

                    let max_y = self.start.y + if line.from.y > line.to.y {line.from.y} else {line.to.y};
                    if bounds.max.y < max_y {
                        bounds.max.y = max_y;
                    }

                    let max_z = if self.start.z > self.end_z {self.start.z} else {self.end_z};
                    if bounds.max.z < max_z {
                        bounds.max.z = max_z;
                    }

                    let min_x = self.start.x + if line.from.x < line.to.x {line.from.x} else {line.to.x};
                    if bounds.min.x > min_x {
                        bounds.min.x = min_x;
                    }

                    let min_y = self.start.y + if line.from.y < line.to.y {line.from.y} else {line.to.y};
                    if bounds.min.y > min_y {
                        bounds.min.y = min_y;
                    }

                    let min_z = if self.start.z < self.end_z {self.start.z} else {self.end_z};
                    if bounds.min.z > min_z {
                        bounds.min.z = min_z;
                    }
                }
            }
        }

        bounds
    }

    #[must_use]
    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        let mut instructions = vec![];

        if self.segments.is_empty() {
            return instructions;
        }

        let start = match &self.segments[0] {
            Segment::Arc(arc) => Vector3 {
                x: arc.from.x + self.start.x,
                y: arc.from.y + self.start.y,
                z: self.start.z,
            },
            Segment::Line(line) => Vector3 {
                x: line.from.x + self.start.x,
                y: line.from.y + self.start.y,
                z: self.start.z,
            },
        };

        instructions.append(&mut vec![
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment {
                text: format!(
                    "Cut path at: x = {}, y = {}",
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

        let mut total_distance = 0.0;
        let mut last_point = Vector2 { x: 0.0, y: 0.0 };

        let mut distances: Vec<f64> = vec![];
        for segment in self.segments.iter() {
            let end = match segment {
                Segment::Arc(arc) => arc.to,
                Segment::Line(line) => line.to,
            };
            let distance = last_point.distance_to(end);
            distances.push(distance);
            total_distance += distance;
            last_point = end;
        }

        let max_step_z = self.max_step_z.abs();

        let layers = ((self.start.z - self.end_z) / max_step_z).floor() as u32;
        let mut start_z = self.start.z;

        for _layer in 0..layers {
            let end_z = start_z - max_step_z;

            instructions.append(&mut self.segments_to_instructions(
                start_z,
                end_z,
                &distances,
                total_distance,
            ));

            start_z = end_z;
        }

        instructions.append(&mut self.segments_to_instructions(
            self.end_z,
            self.end_z,
            &distances,
            total_distance,
        ));

        instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(context.z_safe()),
        }));

        instructions
    }

    fn segments_to_instructions(
        &self,
        start_z: f64,
        end_z: f64,
        distances: &[f64],
        total_distance: f64,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let mut from_z = start_z;

        for (index, segment) in self.segments.iter().enumerate() {
            let distance = distances[index];
            let to_z = from_z - distance / total_distance * (start_z - end_z);

            match segment {
                Segment::Arc(arc) => {
                    instructions.push(Instruction::G1(G1 {
                        x: Some(self.start.x + arc.from.x),
                        y: Some(self.start.y + arc.from.y),
                        z: Some(from_z),
                        f: None,
                    }));

                    instructions.push(Instruction::G2(G2 {
                        x: Some(self.start.x + arc.to.x),
                        y: Some(self.start.y + arc.to.y),
                        z: Some(to_z),
                        i: Some(arc.center.x - arc.from.x),
                        j: Some(arc.center.y - arc.from.y),
                        k: None,
                        r: None,
                        p: None,
                        f: None,
                    }));
                }
                Segment::Line(line) => {
                    instructions.push(Instruction::G1(G1 {
                        x: Some(self.start.x + line.from.x),
                        y: Some(self.start.y + line.from.y),
                        z: Some(from_z),
                        f: None,
                    }));

                    instructions.push(Instruction::G1(G1 {
                        x: Some(self.start.x + line.to.x),
                        y: Some(self.start.y + line.to.y),
                        z: Some(to_z),
                        f: None,
                    }));
                }
            }

            from_z = to_z;
        }

        instructions
    }
}
