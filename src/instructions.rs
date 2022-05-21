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
    pub fn to_gcode(&self) -> String {
        let mut command = "G0".to_string();

        if let Some(x) = self.x {
            command.push_str(&format!(" X{}", round_precision(x)));
        }

        if let Some(y) = self.y {
            command.push_str(&format!(" Y{}", round_precision(y)));
        }

        if let Some(z) = self.z {
            command.push_str(&format!(" Z{}", round_precision(z)));
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
    pub fn to_gcode(&self) -> String {
        let mut command = "G1".to_string();

        if let Some(x) = self.x {
            command.push_str(&format!(" X{}", round_precision(x)));
        }

        if let Some(y) = self.y {
            command.push_str(&format!(" Y{}", round_precision(y)));
        }

        if let Some(z) = self.z {
            command.push_str(&format!(" Z{}", round_precision(z)));
        }

        if let Some(f) = self.f {
            command.push_str(&format!(" F{}", round_precision(f)));
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
    pub fn to_gcode(&self) -> String {
        let mut command = "G2".to_string();

        if let Some(x) = self.x {
            command.push_str(&format!(" X{}", round_precision(x)));
        }

        if let Some(y) = self.y {
            command.push_str(&format!(" Y{}", round_precision(y)));
        }

        if let Some(z) = self.z {
            command.push_str(&format!(" Z{}", round_precision(z)));
        }

        if let Some(r) = self.r {
            command.push_str(&format!(" R{}", round_precision(r)));
        } else {
            if let Some(i) = self.i {
                command.push_str(&format!(" I{}", round_precision(i)));
            }

            if let Some(j) = self.j {
                command.push_str(&format!(" J{}", round_precision(j)));
            }

            if let Some(k) = self.k {
                command.push_str(&format!(" K{}", round_precision(k)));
            }
        }

        if let Some(p) = self.p {
            command.push_str(&format!(" P{}", p));
        }

        if let Some(f) = self.f {
            command.push_str(&format!(" F{}", round_precision(f)));
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
    pub fn to_gcode(&self) -> String {
        let mut command = "G3".to_string();

        if let Some(x) = self.x {
            command.push_str(&format!(" X{}", round_precision(x)));
        }

        if let Some(y) = self.y {
            command.push_str(&format!(" Y{}", round_precision(y)));
        }

        if let Some(z) = self.z {
            command.push_str(&format!(" Z{}", round_precision(z)));
        }

        if let Some(r) = self.r {
            command.push_str(&format!(" R{}", round_precision(r)));
        } else {
            if let Some(i) = self.i {
                command.push_str(&format!(" I{}", round_precision(i)));
            }

            if let Some(j) = self.j {
                command.push_str(&format!(" J{}", round_precision(j)));
            }

            if let Some(k) = self.k {
                command.push_str(&format!(" K{}", round_precision(k)));
            }
        }

        if let Some(p) = self.p {
            command.push_str(&format!(" P{}", p));
        }

        if let Some(f) = self.f {
            command.push_str(&format!(" F{}", round_precision(f)));
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
    pub fn to_gcode(&self) -> String {
        format!("G4 P{}", round_precision(self.p.as_secs_f64()))
    }
}

/// Inch Units
#[derive(Debug, Clone, PartialEq)]
pub struct G20 {}

impl G20 {
    pub fn to_gcode(&self) -> String {
        "G20".to_string()
    }
}

/// Millimeter Units
#[derive(Debug, Clone, PartialEq)]
pub struct G21 {}

impl G21 {
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
    pub fn to_gcode(&self) -> String {
        format!("S{}", round_precision(self.x))
    }
}

/// Program Pause (user must resume)
#[derive(Debug, Clone, PartialEq)]
pub struct M0 {}

impl M0 {
    pub fn to_gcode(&self) -> String {
        "M0".to_string()
    }
}

/// Program End (stop spindle and reset all offsets)
#[derive(Debug, Clone, PartialEq)]
pub struct M2 {}

impl M2 {
    pub fn to_gcode(&self) -> String {
        "M2".to_string()
    }
}

/// Start Spindle (clockwise)
#[derive(Debug, Clone, PartialEq)]
pub struct M3 {}

impl M3 {
    pub fn to_gcode(&self) -> String {
        "M3".to_string()
    }
}

/// Start Spindle (counterclockwise)
#[derive(Debug, Clone, PartialEq)]
pub struct M4 {}

impl M4 {
    pub fn to_gcode(&self) -> String {
        "M4".to_string()
    }
}

/// Stop Spindle
#[derive(Debug, Clone, PartialEq)]
pub struct M5 {}

impl M5 {
    pub fn to_gcode(&self) -> String {
        "M5".to_string()
    }
}

/// Manual Tool Change
#[derive(Debug, Clone, PartialEq)]
pub struct M6 {
    /// Tool number
    pub t: u32,
}

impl M6 {
    pub fn to_gcode(&self) -> String {
        format!("T{} M6", self.t)
    }
}

/// Empty Line
#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl Empty {
    pub fn to_gcode(&self) -> String {
        "".to_string()
    }
}

/// Comment
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Comment
    pub text: String,
}

impl Comment {
    pub fn to_gcode(&self) -> String {
        format!("({})", self.text)
    }
}

/// Message to print
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Message
    pub text: String,
}

impl Message {
    pub fn to_gcode(&self) -> String {
        format!("(MSG,{})", self.text)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    G0(G0),
    G1(G1),
    G2(G2),
    G3(G3),
    G4(G4),
    G20(G20),
    G21(G21),
    G43(G43),
    F(F),
    S(S),
    M0(M0),
    M2(M2),
    M3(M3),
    M4(M4),
    M5(M5),
    M6(M6),
    Empty(Empty),
    Comment(Comment),
    Message(Message),
}

impl Instruction {
    pub fn to_gcode(&self) -> String {
        match self {
            Instruction::G0(instruction) => instruction.to_gcode(),
            Instruction::G1(instruction) => instruction.to_gcode(),
            Instruction::G2(instruction) => instruction.to_gcode(),
            Instruction::G3(instruction) => instruction.to_gcode(),
            Instruction::G4(instruction) => instruction.to_gcode(),
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
