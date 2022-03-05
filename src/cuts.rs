use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Line {
    pub from: Vector2,
    pub to: Vector2,
}

impl Line {
    pub fn new(from: Vector2, to: Vector2) -> Self {
        Self { from, to }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Cut {
    Line(Line),
}
