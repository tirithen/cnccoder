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

impl Tool {
    #[must_use]
    pub fn cylindrical(
        units: Units,
        length: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Tool {
        Tool::Cylindrical(Cylindrical::new(
            units,
            length,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        ))
    }

    #[must_use]
    pub fn ballnose(
        units: Units,
        length: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Tool {
        Tool::Ballnose(Ballnose::new(
            units,
            length,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        ))
    }

    #[must_use]
    pub fn conical(
        units: Units,
        angle: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Tool {
        Tool::Conical(Conical::new(
            units,
            angle,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        ))
    }

    #[must_use]
    pub fn units(&self) -> Units {
        match self {
            Self::Cylindrical(t) => t.units,
            Self::Ballnose(t) => t.units,
            Self::Conical(t) => t.units,
        }
    }

    #[must_use]
    pub fn diameter(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.diameter,
            Self::Ballnose(t) => t.diameter,
            Self::Conical(t) => t.diameter,
        }
    }

    #[must_use]
    pub fn radius(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.diameter / 2.0,
            Self::Ballnose(t) => t.diameter / 2.0,
            Self::Conical(t) => t.diameter / 2.0,
        }
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        match self {
            Self::Cylindrical(t) => t.direction,
            Self::Ballnose(t) => t.direction,
            Self::Conical(t) => t.direction,
        }
    }

    #[must_use]
    pub fn spindle_speed(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.spindle_speed,
            Self::Ballnose(t) => t.spindle_speed,
            Self::Conical(t) => t.spindle_speed,
        }
    }

    #[must_use]
    pub fn feed_rate(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.feed_rate,
            Self::Ballnose(t) => t.feed_rate,
            Self::Conical(t) => t.feed_rate,
        }
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let description = match self {
            Self::Cylindrical(t) => t.to_string(),
            Self::Ballnose(t) => t.to_string(),
            Self::Conical(t) => t.to_string(),
        };

        write!(formatter, "{}", description)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Cylindrical {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
    pub direction: Direction,
    pub spindle_speed: f64,
    pub feed_rate: f64,
}

impl Cylindrical {
    #[must_use]
    pub fn new(
        units: Units,
        length: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Cylindrical {
        Cylindrical {
            units,
            length,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        }
    }

    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Cylindrical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Cylindrical tool diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {}, feed_rate = {}{}/min",
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units.clone(),
            self.direction,
            round_precision(self.spindle_speed),
            round_precision(self.feed_rate),
            units,
        )
    }
}

impl PartialEq for Cylindrical {
    fn eq(&self, other: &Cylindrical) -> bool {
        self.units == other.units
            && self.length == other.length
            && self.diameter == other.diameter
            && self.direction == other.direction
            && self.spindle_speed == other.spindle_speed
            && self.feed_rate == other.feed_rate
    }
}

impl Eq for Cylindrical {}

impl Hash for Cylindrical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.direction.hash(state);
        self.spindle_speed.to_bits().hash(state);
        self.feed_rate.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Ballnose {
    pub units: Units,
    pub length: f64,
    pub diameter: f64,
    pub direction: Direction,
    pub spindle_speed: f64,
    pub feed_rate: f64,
}

impl Ballnose {
    #[must_use]
    pub fn new(
        units: Units,
        length: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Ballnose {
        Ballnose {
            units,
            length,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        }
    }

    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Ballnose {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Ballnose tool diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {}, feed_rate = {}{}/min",
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units.clone(),
            self.direction,
            round_precision(self.spindle_speed),
            round_precision(self.feed_rate),
            units,
        )
    }
}

impl PartialEq for Ballnose {
    fn eq(&self, other: &Ballnose) -> bool {
        self.units == other.units
            && self.length == other.length
            && self.diameter == other.diameter
            && self.direction == other.direction
            && self.spindle_speed == other.spindle_speed
            && self.feed_rate == other.feed_rate
    }
}

impl Eq for Ballnose {}

impl Hash for Ballnose {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.direction.hash(state);
        self.spindle_speed.to_bits().hash(state);
        self.feed_rate.to_bits().hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Conical {
    pub units: Units,
    pub length: f64,
    pub angle: f64,
    pub diameter: f64,
    pub direction: Direction,
    pub spindle_speed: f64,
    pub feed_rate: f64,
}

impl Conical {
    #[must_use]
    pub fn new(
        units: Units,
        angle: f64,
        diameter: f64,
        direction: Direction,
        spindle_speed: f64,
        feed_rate: f64,
    ) -> Conical {
        Conical {
            units,
            length: (diameter / 2.0) / (angle / 2.0).to_radians().tan(),
            angle,
            diameter,
            direction,
            spindle_speed,
            feed_rate,
        }
    }

    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl fmt::Display for Conical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = self.units.to_string();

        write!(
            formatter,
            "Conical tool angle = {}°, diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {}, feed_rate = {}{}/min",
            round_precision(self.angle),
            round_precision(self.diameter),
            units.clone(),
            round_precision(self.length),
            units.clone(),
            self.direction,
            round_precision(self.spindle_speed),
            round_precision(self.feed_rate),
            units,
        )
    }
}

impl PartialEq for Conical {
    fn eq(&self, other: &Conical) -> bool {
        self.units == other.units
            && self.angle == other.angle
            && self.length == other.length
            && self.diameter == other.diameter
            && self.direction == other.direction
            && self.spindle_speed == other.spindle_speed
            && self.feed_rate == other.feed_rate
    }
}

impl Eq for Conical {}

impl Hash for Conical {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.units.hash(state);
        self.angle.to_bits().hash(state);
        self.length.to_bits().hash(state);
        self.diameter.to_bits().hash(state);
        self.direction.hash(state);
        self.spindle_speed.to_bits().hash(state);
        self.feed_rate.to_bits().hash(state);
    }
}
