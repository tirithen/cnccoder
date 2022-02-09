use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::types::Units;
use crate::utils::round_precision;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(tag = "shape", rename_all = "lowercase")]
pub enum Tool {
    Cylindrical(Cylindrical),
    Ballnose(Ballnose),
    Conical(Conical),
}

impl Tool {
    pub fn fake() -> Self {
        Self::Cylindrical(Cylindrical {
            diameter: -1.0,
            length: -1.0,
            ..Default::default()
        })
    }

    pub fn to_string_description(&self) -> String {
        match self {
            Self::Cylindrical(tool) => format!(
                "Cylindrical: diameter = {}{}, length = {}{}",
                round_precision(tool.diameter),
                tool.units.to_string_base_unit(),
                round_precision(tool.length),
                tool.units.to_string_base_unit()
            ),
            Self::Ballnose(tool) => format!(
                "Ballnose: diameter = {}{}, length = {}{}",
                round_precision(tool.diameter),
                tool.units.to_string_base_unit(),
                round_precision(tool.length),
                tool.units.to_string_base_unit()
            ),
            Self::Conical(tool) => format!(
                "Conical: angle = {}°, diameter = {}{}, length = {}{}",
                round_precision(tool.angle),
                round_precision(tool.diameter),
                tool.units.to_string_base_unit(),
                round_precision(tool.length),
                tool.units.to_string_base_unit()
            ),
        }
    }

    pub fn radius(&self) -> f64 {
        match self {
            Self::Cylindrical(tool) => tool.radius(),
            Self::Ballnose(tool) => tool.radius(),
            Self::Conical(tool) => tool.radius(),
        }
    }

    pub fn spindle_rpm(&self) -> f64 {
        match self {
            Self::Cylindrical(tool) => tool.spindle_rpm,
            Self::Ballnose(tool) => tool.spindle_rpm,
            Self::Conical(tool) => tool.spindle_rpm,
        }
    }
}

impl Default for Tool {
    fn default() -> Self {
        Self::Cylindrical(Cylindrical::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Cylindrical {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
    pub spindle_rpm: f64,
}

impl Cylindrical {
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Cylindrical {
    fn default() -> Self {
        Self {
            units: Units::default(),
            length: 10.0,
            diameter: 4.0,
            spindle_rpm: 5000.0,
        }
    }
}

impl PartialEq for Cylindrical {
    fn eq(&self, other: &Cylindrical) -> bool {
        self.units == other.units &&
        self.length == other.length &&
        self.diameter == other.diameter &&
        self.spindle_rpm == other.spindle_rpm
    }
}

impl Eq for Cylindrical {}

impl Hash for Cylindrical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.spindle_rpm.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Ballnose {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
    pub spindle_rpm: f64
}

impl Ballnose {
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Ballnose {
    fn default() -> Self {
        Self {
            units: Units::default(),
            length: 10.0,
            diameter: 4.0,
            spindle_rpm: 5000.0,
        }
    }
}

impl PartialEq for Ballnose {
    fn eq(&self, other: &Ballnose) -> bool {
        self.units == other.units &&
        self.length == other.length &&
        self.diameter == other.diameter &&
        self.spindle_rpm == other.spindle_rpm
    }
}

impl Eq for Ballnose {}

impl Hash for Ballnose {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.spindle_rpm.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Conical {
    pub units: Units,
    pub angle: f64,
    pub length: f64,
    pub diameter: f64,
    pub spindle_rpm: f64,
}

impl Conical {
    pub fn new(units: Units, angle: f64, diameter: f64) -> Self {
        Self {
            units,
            angle,
            length: (diameter / 2.0) / (angle / 2.0).to_radians().tan(),
            diameter,
            spindle_rpm: 5000.0,
        }
    }

    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Conical {
    fn default() -> Self {
        Self::new(Units::default(), 90.0, 4.0)
    }
}

impl PartialEq for Conical {
    fn eq(&self, other: &Conical) -> bool {
        self.units == other.units &&
        self.angle == other.angle &&
        self.length == other.length &&
        self.diameter == other.diameter &&
        self.spindle_rpm == other.spindle_rpm
    }
}

impl Eq for Conical {}

impl Hash for Conical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.angle.to_bits().hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.spindle_rpm.to_bits().hash(state);
    }
}
