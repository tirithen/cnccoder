use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Clone)]
pub struct Line {
    from: Vector2,
    to: Vector2,
}

impl Line {
    pub fn new(from: Vector2, to: Vector2) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone)]
pub enum Segment {
    Line(Line)
}

impl Segment {
    pub fn line(from: Vector2, to: Vector2) -> Self {
        Self::Line(Line::new(from, to))
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
    pub fn new(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self { start, segments, end_z, max_step_z }
    }
}

#[derive(Debug, Clone)]
pub enum Cut {
    Path(Path)
}

impl Cut {
    pub fn path(start: Vector3, segments: Vec<Segment>, end_z: f64, max_step_z: f64) -> Self {
        Self::Path(Path::new(start, segments, end_z, max_step_z))
    }
}