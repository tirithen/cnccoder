use std::{
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::utils::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Tool {
    Cylindrical(Cylindrical),
    Ballnose(Ballnose),
    Conical(Conical),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Cylindrical {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
}

impl Cylindrical {
    pub fn new(units: Units, length: f64, diameter: f64) -> Cylindrical {
        Cylindrical {
            units,
            length,
            diameter,
        }
    }

    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Cylindrical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Cylindrical tool: diameter = {}{}, length = {}{}",
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units
        )
    }
}

impl PartialEq for Cylindrical {
    fn eq(&self, other: &Cylindrical) -> bool {
        self.units == other.units && self.length == other.length && self.diameter == other.diameter
    }
}

impl Eq for Cylindrical {}

impl Hash for Cylindrical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Ballnose {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
}

impl Ballnose {
    pub fn new(units: Units, length: f64, diameter: f64) -> Ballnose {
        Ballnose {
            units,
            length,
            diameter,
        }
    }

    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Ballnose {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Ballnose tool: diameter = {}{}, length = {}{}",
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units
        )
    }
}

impl PartialEq for Ballnose {
    fn eq(&self, other: &Ballnose) -> bool {
        self.units == other.units && self.length == other.length && self.diameter == other.diameter
    }
}

impl Eq for Ballnose {}

impl Hash for Ballnose {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Conical {
    pub length: f64,
    pub units: Units,
    pub angle: f64,
    pub diameter: f64,
}

impl Conical {
    pub fn new(units: Units, angle: f64, diameter: f64) -> Conical {
        Conical {
            length: (diameter / 2.0) / (angle / 2.0).to_radians().tan(),
            units,
            angle,
            diameter,
        }
    }

    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Conical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Conical: angle = {}°, diameter = {}{}, length = {}{}",
            round_precision(self.angle),
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units
        )
    }
}

impl PartialEq for Conical {
    fn eq(&self, other: &Conical) -> bool {
        self.units == other.units
            && self.angle == other.angle
            && self.length == other.length
            && self.diameter == other.diameter
    }
}

impl Eq for Conical {}

impl Hash for Conical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.angle.to_bits().hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
    }
}
