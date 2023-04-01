//! Representations of all supported G-code instructions/commands for cnccoder.
//!
//! This is the lowest level of structs in the crate. Even if they are publicly exposed,
//! they are primarily intended to be used internally by the higher level [cuts](../cuts/index.html).

use std::fmt::Write as _;
use std::time::Duration;

use crate::utils::round_precision;

/// Rapid move
#[derive(Debug, Clone, PartialEq)]
pub struct G0 {
    /// X Coordinate
    pub x: Option<f64>,
    /// Y Coordinate
    pub y: Option<f64>,
    /// Z Coordinate
    pub z: Option<f64>,
}

impl G0 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        let mut command = "G0".to_string();

        if let Some(x) = self.x {
            let _ = write!(command, " X{}", round_precision(x));
        }

        if let Some(y) = self.y {
            let _ = write!(command, " Y{}", round_precision(y));
        }

        if let Some(z) = self.z {
            let _ = write!(command, " Z{}", round_precision(z));
        }

        command
    }
}

/// Linear Move
#[derive(Debug, Clone, PartialEq)]
pub struct G1 {
    /// X Coordinate
    pub x: Option<f64>,
    /// Y Coordinate
    pub y: Option<f64>,
    /// Z Coordinate
    pub z: Option<f64>,
    /// Feedrate
    pub f: Option<f64>,
}

impl G1 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        let mut command = "G1".to_string();

        if let Some(x) = self.x {
            let _ = write!(command, " X{}", round_precision(x));
        }

        if let Some(y) = self.y {
            let _ = write!(command, " Y{}", round_precision(y));
        }

        if let Some(z) = self.z {
            let _ = write!(command, " Z{}", round_precision(z));
        }

        if let Some(f) = self.f {
            let _ = write!(command, " F{}", round_precision(f));
        }

        command
    }
}

/// Arc Move (clockwise)
///
/// Use either R or I, J, mixing all three is not allowed
#[derive(Debug, Clone, PartialEq)]
pub struct G2 {
    /// X Coordinate
    pub x: Option<f64>,
    /// Y Coordinate
    pub y: Option<f64>,
    /// Z Coordinate
    pub z: Option<f64>,
    /// X Offset
    pub i: Option<f64>,
    /// Y Offset
    pub j: Option<f64>,
    /// Z Offset
    pub k: Option<f64>,
    /// Radius
    pub r: Option<f64>,
    ///  Number of complete circles
    pub p: Option<u32>,
    /// Feedrate
    pub f: Option<f64>,
}

impl G2 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        let mut command = "G2".to_string();

        if let Some(x) = self.x {
            let _ = write!(command, " X{}", round_precision(x));
        }

        if let Some(y) = self.y {
            let _ = write!(command, " Y{}", round_precision(y));
        }

        if let Some(z) = self.z {
            let _ = write!(command, " Z{}", round_precision(z));
        }

        if let Some(r) = self.r {
            let _ = write!(command, " R{}", round_precision(r));
        } else {
            if let Some(i) = self.i {
                let _ = write!(command, " I{}", round_precision(i));
            }

            if let Some(j) = self.j {
                let _ = write!(command, " J{}", round_precision(j));
            }

            if let Some(k) = self.k {
                let _ = write!(command, " K{}", round_precision(k));
            }
        }

        if let Some(p) = self.p {
            let _ = write!(command, " P{}", round_precision(p.into()));
        }

        if let Some(f) = self.f {
            let _ = write!(command, " F{}", round_precision(f));
        }

        command
    }
}

/// Arc Move (counterclockwise)
///
/// Use either R or I, J, mixing all three is not allowed
#[derive(Debug, Clone, PartialEq)]
pub struct G3 {
    /// X Coordinate
    pub x: Option<f64>,
    /// Y Coordinate
    pub y: Option<f64>,
    /// Z Coordinate
    pub z: Option<f64>,
    /// X Offset
    pub i: Option<f64>,
    /// Y Offset
    pub j: Option<f64>,
    /// Z Offset
    pub k: Option<f64>,
    /// Radius
    pub r: Option<f64>,
    ///  Number of complete circles
    pub p: Option<u32>,
    /// Feedrate
    pub f: Option<f64>,
}

impl G3 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        let mut command = "G3".to_string();

        if let Some(x) = self.x {
            let _ = write!(command, " X{}", round_precision(x));
        }

        if let Some(y) = self.y {
            let _ = write!(command, " Y{}", round_precision(y));
        }

        if let Some(z) = self.z {
            let _ = write!(command, " Z{}", round_precision(z));
        }

        if let Some(r) = self.r {
            let _ = write!(command, " R{}", round_precision(r));
        } else {
            if let Some(i) = self.i {
                let _ = write!(command, " I{}", round_precision(i));
            }

            if let Some(j) = self.j {
                let _ = write!(command, " J{}", round_precision(j));
            }

            if let Some(k) = self.k {
                let _ = write!(command, " K{}", round_precision(k));
            }
        }

        if let Some(p) = self.p {
            let _ = write!(command, " P{}", round_precision(p.into()));
        }

        if let Some(f) = self.f {
            let _ = write!(command, " F{}", round_precision(f));
        }

        command
    }
}

/// Dwell (pause duration)
#[derive(Debug, Clone, PartialEq)]
pub struct G4 {
    /// Duration to pause (serializedto seconds)
    pub p: Duration,
}

impl G4 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("G4 P{}", round_precision(self.p.as_secs_f64()))
    }
}

/// Select Plane XY
#[derive(Debug, Clone, PartialEq)]
pub struct G17 {}

impl G17 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "G17".to_string()
    }
}

/// Select Plane ZX
#[derive(Debug, Clone, PartialEq)]
pub struct G18 {}

impl G18 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "G18".to_string()
    }
}

/// Select Plane YZ
#[derive(Debug, Clone, PartialEq)]
pub struct G19 {}

impl G19 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "G19".to_string()
    }
}

/// Inch Units
#[derive(Debug, Clone, PartialEq)]
pub struct G20 {}

impl G20 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "G20".to_string()
    }
}

/// Millimeter Units
#[derive(Debug, Clone, PartialEq)]
pub struct G21 {}

impl G21 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "G21".to_string()
    }
}

/// Tool Length Offset (applies offset to all coordinates)
#[derive(Debug, Clone, PartialEq)]
pub struct G43 {
    /// Tool number (offset will be looked up in the tool table)
    pub h: u32,
}

impl G43 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("G43 H{}", self.h)
    }
}

/// Set Feed Rate
#[derive(Debug, Clone, PartialEq)]
pub struct F {
    /// Feed rate (units per minute)
    pub x: f64,
}

impl F {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("F{}", round_precision(self.x))
    }
}

/// Set Spindle Speed
#[derive(Debug, Clone, PartialEq)]
pub struct S {
    /// Feed rate (rpm)
    pub x: f64,
}

impl S {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("S{}", round_precision(self.x))
    }
}

/// Program Pause (user must resume)
#[derive(Debug, Clone, PartialEq)]
pub struct M0 {}

impl M0 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "M0".to_string()
    }
}

/// Program End (stop spindle and reset all offsets)
#[derive(Debug, Clone, PartialEq)]
pub struct M2 {}

impl M2 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "M2".to_string()
    }
}

/// Start Spindle (clockwise)
#[derive(Debug, Clone, PartialEq)]
pub struct M3 {}

impl M3 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "M3".to_string()
    }
}

/// Start Spindle (counterclockwise)
#[derive(Debug, Clone, PartialEq)]
pub struct M4 {}

impl M4 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "M4".to_string()
    }
}

/// Stop Spindle
#[derive(Debug, Clone, PartialEq)]
pub struct M5 {}

impl M5 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "M5".to_string()
    }
}

/// Manual Tool Change
#[derive(Debug, Clone, PartialEq)]
pub struct M6 {
    /// Tool number
    pub t: u8,
}

impl M6 {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("T{} M6", self.t)
    }
}

/// Empty Line
#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl Empty {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        "".to_string()
    }
}

/// Comment
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Comment {
    /// Comment
    pub text: String,
}

impl Comment {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        if self.text.is_empty() {
            return String::new();
        }

        format!(";({})", self.text)
    }
}

/// Message to print
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message
    pub text: String,
}

impl Message {
    /// Generate G-code string
    pub fn to_gcode(&self) -> String {
        format!("(MSG,{})", self.text)
    }
}

/// The Instruction enum is used to represent a single G-code command in a program.
/// See the
/// [Grbl reference](https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands#g---view-gcode-parser-state)
/// for more details.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Command G0, Rapid Move
    G0(G0),
    /// Command G1, Linear Move
    G1(G1),
    /// Command G2, Arc Move (clockwise)
    G2(G2),
    /// Command G3, Arc Move (counterclockwise)
    G3(G3),
    /// Command G4, Dwell
    G4(G4),
    /// Command G17, Select Plane XY
    G17(G17),
    /// Command G18, Select Plane XZ
    G18(G18),
    /// Command G19, Select Plane YZ
    G19(G19),
    /// Command G20, Inch Units
    G20(G20),
    /// Command G20, Millimeter Units
    G21(G21),
    /// Command G43, Tool Length Offset
    G43(G43),
    /// Command F, Set Feed Rate
    F(F),
    /// Command S, Set Spindle Speed
    S(S),
    /// Command M0, Program Pause
    M0(M0),
    /// Command M2, Program End
    M2(M2),
    /// Command M3, Start Spindle (clockwise)
    M3(M3),
    /// Command M4, Start Spindle (counterclockwise)
    M4(M4),
    /// Command M5, Stop Spindle
    M5(M5),
    /// Command M6, Manual Tool Change
    M6(M6),
    /// Command Empty, Empty Line
    Empty(Empty),
    /// Command Comment, Comment
    Comment(Comment),
    /// Command Message, Message to point
    Message(Message),
}

impl Instruction {
    /// Converts instruction to G-code
    pub fn to_gcode(&self) -> String {
        match self {
            Instruction::G0(instruction) => instruction.to_gcode(),
            Instruction::G1(instruction) => instruction.to_gcode(),
            Instruction::G2(instruction) => instruction.to_gcode(),
            Instruction::G3(instruction) => instruction.to_gcode(),
            Instruction::G4(instruction) => instruction.to_gcode(),
            Instruction::G17(instruction) => instruction.to_gcode(),
            Instruction::G18(instruction) => instruction.to_gcode(),
            Instruction::G19(instruction) => instruction.to_gcode(),
            Instruction::G20(instruction) => instruction.to_gcode(),
            Instruction::G21(instruction) => instruction.to_gcode(),
            Instruction::G43(instruction) => instruction.to_gcode(),
            Instruction::F(instruction) => instruction.to_gcode(),
            Instruction::S(instruction) => instruction.to_gcode(),
            Instruction::M0(instruction) => instruction.to_gcode(),
            Instruction::M2(instruction) => instruction.to_gcode(),
            Instruction::M3(instruction) => instruction.to_gcode(),
            Instruction::M4(instruction) => instruction.to_gcode(),
            Instruction::M5(instruction) => instruction.to_gcode(),
            Instruction::M6(instruction) => instruction.to_gcode(),
            Instruction::Empty(instruction) => instruction.to_gcode(),
            Instruction::Comment(instruction) => instruction.to_gcode(),
            Instruction::Message(instruction) => instruction.to_gcode(),
        }
    }
}
