//! Helper module for generating Camotics project files.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{program::*, tools::*, types::*};

/// Resolution mode, when creating a Camotics struct `ResolutionMode::Manual`
/// is used by default to allow setting a custom resolution for the simulation.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionMode {
    /// Corresponds to a resolution of 0.116348.
    High,
    /// Corresponds to a resolution of 0.428631.
    Low,
    /// Allows for custom resolution values to be set.
    Manual,
}

/// Defines the size of the workpiece, when creating a Camotics struct these
/// values are calculated from the program.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Workpiece {
    /// Indicates of bounds should be calculated by Camotics automatically.
    pub automatic: bool,
    /// Extra margin added to the Camotics atumated calculation.
    pub margin: f64,
    /// Manual bounds for the workpiece, will be automatically calculated
    /// from the program.
    pub bounds: Bounds,
}

/// Tool shape, will be derived from the [tools](../tools/index.html) used in the program.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CamoticsToolShape {
    /// Cylindrical tool
    Cylindrical,
    /// Ballnose tool
    Ballnose,
    /// Conical tool
    Conical,
}

impl Default for CamoticsToolShape {
    fn default() -> Self {
        Self::Cylindrical
    }
}

/// Tool definition in the format required by Camotics, will be derived from the
/// [tools](../tools/index.html) used in the program.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct CamoticsTool {
    /// Measurement units of the tool
    pub units: Units,
    /// Angle of a conical tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    /// Cutter length of the tool
    pub length: f64,
    /// Cutter diameter of the tool
    pub diameter: f64,
    /// The tool number/identifier
    pub number: u32,
    /// The shape of the tool
    pub shape: CamoticsToolShape,
}

impl CamoticsTool {
    /// Creates a new `CamoticsTool` from a program [Tool](../tools/enum.Tool.html).
    #[must_use]
    pub fn from_tool(tool: Tool, number: u32) -> Self {
        match tool {
            Tool::Cylindrical(t) => CamoticsTool {
                units: t.units,
                angle: None,
                length: t.length,
                diameter: t.diameter,
                number,
                shape: CamoticsToolShape::Cylindrical,
            },
            Tool::Ballnose(t) => CamoticsTool {
                units: t.units,
                angle: None,
                length: t.length,
                diameter: t.diameter,
                number,
                shape: CamoticsToolShape::Ballnose,
            },
            Tool::Conical(t) => CamoticsTool {
                units: t.units,
                angle: Some(t.angle),
                length: t.length,
                diameter: t.diameter,
                number,
                shape: CamoticsToolShape::Conical,
            },
        }
    }
}

/// Representation for a [Camotics](https://camotics.org/) project file,
/// running `.to_json_string()` outputs a project file that can be opened
/// directly by Camotics.
///
/// To write a camotics file and a gcode file in one go, see
/// [write_project](../filesystem/fn.write_project.html).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Camotics {
    /// The name of the project.
    #[serde(skip_serializing)]
    pub name: String,
    /// The units used by the project.
    pub units: Units,
    /// The resolution mode used by the project, will be `ResolutionMode::Manual`
    /// by default.
    #[serde(rename(serialize = "resolution-mode"))]
    pub resolution_mode: ResolutionMode,
    /// The resolution used for the simulation, a higher value uses more system
    /// memory and takes longer/more CPU to simulate. Suggested value is between
    /// 0.5 and 1.0 depending on the detail value required. A lower value equals
    /// more detail.
    pub resolution: f64,
    /// Tools used in the project, when using
    /// [Camotics::from_program](struct.Camotics.html#method.new) the
    /// program tools passed in will be converted to `CamoticsTool` instances.
    pub tools: HashMap<u32, CamoticsTool>,
    /// The size of the workpiece for the project.
    pub workpiece: Workpiece,
    /// The G-code files used by this project. When using
    /// [Camotics::from_program](struct.Camotics.html#method.new)
    /// the program G-code filename will be added from the name argument.
    pub files: Vec<String>,
}

impl Camotics {
    /// Creates a new `Camotics` project struct from a name, program tools, bounds, and resolution.
    #[must_use]
    pub fn new(name: &str, tools: Vec<Tool>, workpiece: Bounds, resolution: f64) -> Self {
        let mut tools_map = HashMap::new();

        for (index, tool) in tools.iter().enumerate() {
            let number = (index + 1) as u32;
            tools_map.insert(number, CamoticsTool::from_tool(*tool, number));
        }

        Self {
            name: name.to_string(),
            units: Units::Metric,
            resolution_mode: ResolutionMode::Manual,
            resolution,
            tools: tools_map,
            workpiece: Workpiece {
                automatic: false,
                margin: 0.0,
                bounds: workpiece,
            },
            files: vec![format!("{}.gcode", name)],
        }
    }

    /// Creates a new `Camotics` struct from a name, program, and resolution.
    #[must_use]
    pub fn from_program(name: &str, program: Program, resolution: f64) -> Self {
        let tools = program.tools();
        let workpiece = program.bounds();
        Self::new(name, tools, workpiece, resolution)
    }

    /// Serializes the Camotics struct to the JSON format used by the Camotics
    /// application when loading a project.
    #[must_use]
    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::Value;

    use super::*;
    use crate::cuts::*;

    #[test]
    fn test_serialization() {
        let mut tools = HashMap::new();

        tools.insert(
            1,
            CamoticsTool {
                number: 1,
                angle: None,
                units: Units::Metric,
                shape: CamoticsToolShape::Cylindrical,
                length: 50.0,
                diameter: 4.0,
            },
        );

        let camotics = Camotics {
            name: "testing".to_string(),
            units: Units::Metric,
            resolution_mode: ResolutionMode::Manual,
            resolution: 0.3,
            tools,
            workpiece: Workpiece {
                automatic: false,
                margin: 5.0,
                bounds: Bounds {
                    min: Vector3 {
                        x: -60.5,
                        y: -60.5,
                        z: -3.0,
                    },
                    max: Vector3 {
                        x: 119.5,
                        y: 60.5,
                        z: 0.0,
                    },
                },
            },
            files: vec!["file.gcode".to_string()],
        };

        let serialized = serde_json::to_string(&camotics).unwrap();
        let output: Value = serde_json::from_str(&serialized).unwrap();

        let expected: Value = serde_json::from_str(
            r#"
            {
                "units": "metric",
                "resolution-mode": "manual",
                "resolution": 0.3,
                "tools": {
                    "1": {
                        "number": 1,
                        "units": "metric",
                        "shape": "cylindrical",
                        "length": 50.0,
                        "diameter": 4.0
                    }
                },
                "workpiece": {
                    "automatic": false,
                    "margin": 5.0,
                    "bounds": {
                        "min": [-60.5, -60.5, -3.0],
                        "max": [119.5, 60.5, 0.0]
                    }
                },
                "files": [
                    "file.gcode"
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_camotics_from_program() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        program.extend(tool, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(
                    Vector2::default(),
                    Vector2::new(-28.0, -30.0),
                )],
                -0.1,
                1.0,
            ));

            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![
                    Segment::line(Vector2::new(23.0, 12.0), Vector2::new(5.0, 10.0)),
                    Segment::line(Vector2::new(5.0, 10.0), Vector2::new(67.0, 102.0)),
                    Segment::line(Vector2::new(67.0, 102.0), Vector2::new(23.0, 12.0)),
                ],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        let camotics = Camotics::from_program("test-project", program.clone(), 1.0);

        let mut tools = HashMap::new();
        tools.insert(1, CamoticsTool::from_tool(tool, 1));

        assert_eq!(
            camotics,
            Camotics {
                name: "test-project".to_string(),
                units: Units::Metric,
                resolution_mode: ResolutionMode::Manual,
                resolution: 1.0,
                tools,
                workpiece: Workpiece {
                    automatic: false,
                    margin: 0.0,
                    bounds: program.bounds()
                },
                files: vec!["test-project.gcode".to_string()]
            }
        );

        Ok(())
    }
}
