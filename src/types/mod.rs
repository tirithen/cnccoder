use std::fmt;

use serde::{Deserialize, Serialize};

mod vector;
pub use vector::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Bounds {
    pub min: Vector3,
    pub max: Vector3,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Vector3::default(),
            max: Vector3::default(),
        }
    }
}

impl Bounds {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(x, y, z),
        }
    }

    #[must_use]
    pub fn minmax() -> Self {
        Self {
            min: Vector3::max(),
            max: Vector3::min(),
        }
    }

    #[must_use]
    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    Metric,
    Imperial,
}

impl Units {
    pub fn mm_to_inch(mm: f64) -> f64 {
        mm * 25.4
    }

    pub fn measurement_from_mm(self, value: f64) -> f64 {
        match self {
            Self::Metric => value,
            Self::Imperial => Self::mm_to_inch(value),
        }
    }

    pub fn default_z_end(self) -> f64 {
        match self {
            Self::Metric => 0.1,
            Self::Imperial => Self::mm_to_inch(0.1),
        }
    }

    pub fn default_z_max_step(self) -> f64 {
        match self {
            Self::Metric => 1.0,
            Self::Imperial => Self::mm_to_inch(1.0),
        }
    }
}

impl fmt::Display for Units {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Units::Metric => "mm",
                Units::Imperial => "\"",
            }
        )
    }
}

impl Default for Units {
    fn default() -> Self {
        Units::Metric
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Clockwise,
    Counterclockwise,
}

impl fmt::Display for Direction {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Direction::Clockwise => "clockwise",
                Direction::Counterclockwise => "counterclockwise",
            }
        )
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Clockwise
    }
}

#[derive(Debug, Clone)]
pub enum Axis {
    X,
    Z,
    Y,
}

impl fmt::Display for Axis {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Axis::X => "X",
                Axis::Y => "Y",
                Axis::Z => "Z",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum ToolPathCompensation {
    None,
    Inner,
    Outer,
}

impl fmt::Display for ToolPathCompensation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                ToolPathCompensation::None => "none",
                ToolPathCompensation::Inner => "inner",
                ToolPathCompensation::Outer => "outer",
            }
        )
    }
}

impl Default for ToolPathCompensation {
    fn default() -> Self {
        ToolPathCompensation::None
    }
}
