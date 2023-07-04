//! Shared types used by cnccoder, such as Vector2, Vector3, Units, Direction, Axis and Bounds.

use std::fmt;

use serde::{Deserialize, Serialize};

mod vector;
pub use vector::*;

/// Represents an area in 3D space from one min and one max [Vector3](struct.Vector3.html) point.
#[derive(Default, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Bounds {
    /// Minimum 3D point of the boundary.
    pub min: Vector3,
    /// Maximum 3D point of the boundary.
    pub max: Vector3,
}

impl Bounds {
    /// Create a `Bounds` struct starting at xyz 0.0 and ending at the given xyz coordinates.
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(x, y, z),
        }
    }

    /// Create a `Bounds` struct using xyz `f64::MIN` as min and `f64::MAX` for max.
    #[must_use]
    pub fn minmax() -> Self {
        Self {
            min: Vector3::max(),
            max: Vector3::min(),
        }
    }

    /// Return the size of the bounds area as a [Vector3](struct.Vector3.html).
    #[must_use]
    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

/// Indicates if metric or imperial units should be used. This is used as a setting both for a
/// [Program](../program/struct.Program.html) and [tools](../tools/).
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    /// Indicates that measurements are given using millimeters.
    #[default]
    Metric,
    /// Indicates that measurements are given using inches.
    Imperial,
}

impl Units {
    /// Converts from millimeters to inches
    pub fn mm_to_inch(mm: f64) -> f64 {
        mm * 25.4
    }

    /// Converts a measurement from the selected unit to millimeters
    pub fn measurement_from_mm(self, value: f64) -> f64 {
        match self {
            Self::Metric => value,
            Self::Imperial => Self::mm_to_inch(value),
        }
    }

    /// Provides default z_end value, what the CNC should consider as the vertical bottom value
    /// by default, either as millimeters or inches.
    pub fn default_z_end(self) -> f64 {
        match self {
            Self::Metric => 0.1,
            Self::Imperial => Self::mm_to_inch(0.1),
        }
    }

    /// Provides default z_max_step value. As the CNC machine cuts a path it will often be
    /// instructed to cut the path in several passes instead of cutting the full depth at once.
    /// This function provides a default value as millimeters or inches.
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

/// Indicates a rotation direction, this is used by the [tools](../tools/), but also when cutting [arcs](../cuts/struct.Arc.html).
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Clockwise direction.
    #[default]
    Clockwise,
    /// Counter clockwise direction.
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

/// Indicates one specific axis, mainy when cutting [arcs](../cuts/struct.Arc.html).
#[derive(Debug, Clone)]
pub enum Axis {
    /// Indicates X axis.
    X,
    /// Indicates Y axis.
    Z,
    /// Indicates Z axis.
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

/// Indicates how a path should be compensated by the radius of the tool.
#[derive(Debug, Clone, Default)]
pub enum ToolPathCompensation {
    /// The tool will cut at the specified path, without compensating for the radius. This is the default value.
    #[default]
    None,
    /// The tool will cut at the inside of the path, this is useful for pocket cuts.
    Inner,
    /// The tool will cut at the outside of the path, this is useful for contour/frame cuts.
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
