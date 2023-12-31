#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("ballnose-tool", "doc-assets/ballnose-tool.webp"),
doc = ::embed_doc_image::embed_image!("conical-tool", "doc-assets/conical-tool.webp"),
doc = ::embed_doc_image::embed_image!("cylindrical-tool", "doc-assets/cylindrical-tool.webp"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile with feature `doc-images` and Rust version >= 1.54 \
           to enable."
)]

//! Module containing tool configurations for ballnose, conical, and cylindrical cutting tools.
//!
//! |Tool type |Example image |Uses |
//! |--- |---  |--- |
//! |Ballnose|![Ballnose router tool][ballnose-tool]|The ballnose tool can be useful for 3D carving to achieve a smooth surface result.|
//! |Conical|![90° Conical router tool][conical-tool]|The conical tool can be useful for v carving when engraving images, inlays or text.|
//! |Cylindrical|![Cylindrical router tool][cylindrical-tool]|The cylindrical tool is a great general purpose tool when cutting contours, pockets, holes, and planing.|
//!
//! Creating a tool using the `Tool::cylindrical` helper:
//! ```
//! use cnccoder::prelude::*;
//!
//! // Create a cylindrical tool
//! let tool = Tool::cylindrical(
//!     Units::Metric, // Unit for tool measurements
//!     20.0, // Cutter length
//!     10.0, // Cutter diameter
//!     Direction::Clockwise, // Spindle rotation direction
//!     20000.0, // Spindle speed (rpm)
//!     5000.0, // Max feed rate/speed that the cutter will travel with (mm/min)
//! );
//! ```

use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::utils::*;

/// Represents a tool configuration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Tool {
    /// Ballnose tool configuration.
    Ballnose(Ballnose),
    /// Conical tool configuration.
    Conical(Conical),
    /// Cylindrical tool configuration.
    Cylindrical(Cylindrical),
}

impl Tool {
    /// Helper for creating a ballnose tool configuration.
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

    /// Helper for creating a conical tool configuration.
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

    /// Helper for creating a cylindrical tool configuration.
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

    /// Returns the units used for the tool measurements (mm for metric, and inches for imperial).
    #[must_use]
    pub fn units(&self) -> Units {
        match self {
            Self::Cylindrical(t) => t.units,
            Self::Ballnose(t) => t.units,
            Self::Conical(t) => t.units,
        }
    }

    /// Returns the diameter of the tool cutter.
    #[must_use]
    pub fn diameter(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.diameter,
            Self::Ballnose(t) => t.diameter,
            Self::Conical(t) => t.diameter,
        }
    }

    /// Returns the radius of the tool cutter.
    #[must_use]
    pub fn radius(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.diameter / 2.0,
            Self::Ballnose(t) => t.diameter / 2.0,
            Self::Conical(t) => t.diameter / 2.0,
        }
    }

    /// Returns the spin direction for the tool.
    #[must_use]
    pub fn direction(&self) -> Direction {
        match self {
            Self::Cylindrical(t) => t.direction,
            Self::Ballnose(t) => t.direction,
            Self::Conical(t) => t.direction,
        }
    }

    /// Returns the configured spindle/tool rotation speed (rpm).
    #[must_use]
    pub fn spindle_speed(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.spindle_speed,
            Self::Ballnose(t) => t.spindle_speed,
            Self::Conical(t) => t.spindle_speed,
        }
    }

    /// Returns the configured feed rate (mm/min for metric and inches/min for imperial) for the tool.
    #[must_use]
    pub fn feed_rate(&self) -> f64 {
        match self {
            Self::Cylindrical(t) => t.feed_rate,
            Self::Ballnose(t) => t.feed_rate,
            Self::Conical(t) => t.feed_rate,
        }
    }
}

impl Default for Tool {
    fn default() -> Self {
        Self::Cylindrical(Cylindrical::default())
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

/// Ballnose tool configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Ballnose {
    /// The units used for the tool measurements (mm for metric, and inches for imperial).
    pub units: Units,
    /// The length of the tool cutter.
    pub length: f64,
    /// The diameter of the tool cutter.
    pub diameter: f64,
    /// The spin direction for the tool.
    pub direction: Direction,
    /// The selected spindle/tool rotation speed (rpm) for this tool.
    pub spindle_speed: f64,
    /// The selected feed rate (mm/min for metric and inches/min for imperial) for this tool.
    pub feed_rate: f64,
}

impl Ballnose {
    /// Creates a new `Ballnose` tool struct
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

    /// Returns the radius of the tool cutter.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Ballnose {
    fn default() -> Self {
        Self {
            units: Units::Metric,
            length: 5.0,
            diameter: 2.0,
            direction: Direction::Clockwise,
            spindle_speed: 10000.0,
            feed_rate: 500.0,
        }
    }
}

impl fmt::Display for Ballnose {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = match self.units {
            Units::Imperial => self.units.to_string(),
            Units::Metric => format!(" {}", self.units),
        };

        write!(
            formatter,
            "type = Ballnose, diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {} rpm, feed_rate = {}{}/min",
            round_precision(self.diameter),
            units,
            round_precision(self.length),
            units,
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

/// Conical tool configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Conical {
    /// The units used for the tool measurements (mm for metric, and inches for imperial).
    pub units: Units,
    /// The length of the tool cutter.
    pub length: f64,
    /// The angle of the tool cutter.
    pub angle: f64,
    /// The diameter of the tool cutter.
    pub diameter: f64,
    /// The spin direction for the tool.
    pub direction: Direction,
    /// The selected spindle/tool rotation speed (rpm) for this tool.
    pub spindle_speed: f64,
    /// The selected feed rate (mm/min for metric and inches/min for imperial) for this tool.
    pub feed_rate: f64,
}

impl Conical {
    /// Creates a new `Conical` tool struct
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

    /// Returns the radius of the tool cutter.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Conical {
    fn default() -> Self {
        Self {
            units: Units::Metric,
            length: 8.0,
            angle: 90.0,
            diameter: 16.0,
            direction: Direction::Clockwise,
            spindle_speed: 10000.0,
            feed_rate: 500.0,
        }
    }
}

impl fmt::Display for Conical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = match self.units {
            Units::Imperial => self.units.to_string(),
            Units::Metric => format!(" {}", self.units),
        };

        write!(
            formatter,
            "type = Conical, angle = {}°, diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {} rpm, feed_rate = {}{}/min",
            round_precision(self.angle),
            round_precision(self.diameter),
            units,
            round_precision(self.length),
            units,
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

/// Cylindrical tool configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Cylindrical {
    /// The units used for the tool measurements (mm for metric, and inches for imperial).
    pub units: Units,
    /// The length of the tool cutter.
    pub length: f64,
    /// The diameter of the tool cutter.
    pub diameter: f64,
    /// The spin direction for the tool.
    pub direction: Direction,
    /// The selected spindle/tool rotation speed (rpm) for this tool.
    pub spindle_speed: f64,
    /// The selected feed rate (mm/min for metric and inches/min for imperial) for this tool.
    pub feed_rate: f64,
}

impl Cylindrical {
    /// Creates a new `Cylindrical` tool struct
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

    /// Returns the radius of the tool cutter.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.diameter / 2.0
    }
}

impl Default for Cylindrical {
    fn default() -> Self {
        Self {
            units: Units::Metric,
            length: 30.0,
            diameter: 6.0,
            direction: Direction::Clockwise,
            spindle_speed: 10000.0,
            feed_rate: 500.0,
        }
    }
}

impl fmt::Display for Cylindrical {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let units = match self.units {
            Units::Imperial => self.units.to_string(),
            Units::Metric => format!(" {}", self.units),
        };

        write!(
            formatter,
            "type = Cylindrical, diameter = {}{}, length = {}{}, direction = {}, spindle_speed = {} rpm, feed_rate = {}{}/min",
            round_precision(self.diameter),
            units,
            round_precision(self.length),
            units,
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

/// Keeps a list of tools and their order. It also allows for manipulating the order of the tools to ease choosing which cuts that should be made first.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ToolOrdering {
    tools: Vec<Tool>,
    ordering: HashMap<Tool, u8>,
    explicit_ordering: HashMap<Tool, u8>,
}

impl ToolOrdering {
    fn next_auto_order(&self) -> u8 {
        let mut next_order = 1;

        while self
            .ordering
            .values()
            .chain(self.explicit_ordering.values())
            .any(|&order| order == next_order)
        {
            next_order += 1;
        }

        next_order
    }

    /// Adds a tool and automatically assign the first available order to the tool and its related cuts.
    pub fn auto_ordering(&mut self, tool: &Tool) {
        if self.ordering.contains_key(tool) {
            return;
        }

        self.tools.push(*tool);
        self.ordering.insert(*tool, self.next_auto_order());
    }

    /// Adds a tool and assign a specific order to the tool and its related cuts. The minimum order value is 1.
    pub fn set_ordering(&mut self, tool: &Tool, order: u8) {
        let order = if order == 0 { 1 } else { order };

        self.tools.push(*tool);
        self.explicit_ordering
            .retain(|t, o| *o != order || t == tool);
        self.explicit_ordering.insert(*tool, order);

        self.ordering.clear();

        self.explicit_ordering.iter().for_each(|(t, &o)| {
            self.ordering.insert(*t, o);
        });

        for tool in &self.tools {
            if !self.ordering.contains_key(tool) {
                self.ordering.insert(*tool, self.next_auto_order());
            }
        }
    }

    /// Returns the order of the tool and its related cuts, returns None if the tool has not been added.
    pub fn ordering(&self, tool: &Tool) -> Option<u8> {
        self.ordering.get(tool).copied()
    }

    /// Returns an ordered list of the tools added.
    pub fn tools_ordered(&self) -> Vec<Tool> {
        let mut tools = vec![];

        let tool_ordering = &self.ordering;
        let mut orderings: Vec<_> = tool_ordering.iter().collect();
        orderings.sort_by(|a, b| a.1.cmp(b.1));

        for (tool, _) in orderings {
            tools.push(*tool);
        }

        tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_ordering() {
        let mut tool_ordering = ToolOrdering::default();

        let tool1 = Tool::conical(
            Units::Metric,
            30.0,
            6.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );
        let tool2 = Tool::cylindrical(
            Units::Metric,
            30.0,
            2.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );

        tool_ordering.auto_ordering(&tool1);
        tool_ordering.auto_ordering(&tool2);

        assert_eq!(tool_ordering.ordering(&tool1), Some(1));
        assert_eq!(tool_ordering.ordering(&tool2), Some(2));
        assert_ne!(
            tool_ordering.ordering(&tool1),
            tool_ordering.ordering(&tool2)
        );
    }

    #[test]
    fn test_set_ordering() {
        let mut tool_ordering = ToolOrdering::default();

        let tool1 = Tool::conical(
            Units::Metric,
            45.0,
            6.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );
        let tool2 = Tool::cylindrical(
            Units::Metric,
            20.0,
            4.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );

        tool_ordering.set_ordering(&tool1, 1);
        tool_ordering.set_ordering(&tool2, 2);

        assert_eq!(tool_ordering.ordering(&tool1), Some(1));
        assert_eq!(tool_ordering.ordering(&tool2), Some(2));

        tool_ordering.set_ordering(&tool1, 3);

        assert_eq!(tool_ordering.ordering(&tool1), Some(3));
        assert_eq!(tool_ordering.ordering(&tool2), Some(2));
    }

    #[test]
    fn test_mix_set_and_auto_ordering() {
        let mut tool_ordering = ToolOrdering::default();

        let tool1 = Tool::conical(
            Units::Metric,
            30.0,
            4.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );
        let tool2 = Tool::ballnose(
            Units::Metric,
            20.0,
            1.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );
        let tool3 = Tool::cylindrical(
            Units::Metric,
            32.0,
            2.0,
            Direction::Clockwise,
            10000.0,
            500.0,
        );

        tool_ordering.auto_ordering(&tool1);
        tool_ordering.set_ordering(&tool2, 1);
        tool_ordering.auto_ordering(&tool3);

        assert_eq!(tool_ordering.ordering(&tool1), Some(2));
        assert_eq!(tool_ordering.ordering(&tool2), Some(1));
        assert_eq!(tool_ordering.ordering(&tool3), Some(3));

        assert_ne!(
            tool_ordering.ordering(&tool1),
            tool_ordering.ordering(&tool2)
        );
        assert_ne!(
            tool_ordering.ordering(&tool1),
            tool_ordering.ordering(&tool3)
        );
        assert_ne!(
            tool_ordering.ordering(&tool2),
            tool_ordering.ordering(&tool3)
        );
    }
}
