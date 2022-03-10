use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{program::*, tools::*, types::*};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionMode {
    High,
    Low,
    Manual,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Workpiece {
    pub automatic: bool,
    pub margin: f64,
    pub bounds: Bounds,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CamoticsToolShape {
    Cylindrical,
    Ballnose,
    Conical,
}

impl Default for CamoticsToolShape {
    fn default() -> Self {
        Self::Cylindrical
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct CamoticsTool {
    pub units: Units,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    pub length: f64,
    pub diameter: f64,
    pub number: u32,
    pub shape: CamoticsToolShape,
}

impl CamoticsTool {
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Camotics {
    #[serde(skip_serializing)]
    pub name: String,
    pub units: Units,
    #[serde(rename(serialize = "resolution-mode"))]
    pub resolution_mode: ResolutionMode,
    pub resolution: f64,
    pub tools: HashMap<u32, CamoticsTool>,
    pub workpiece: Workpiece,
    pub files: Vec<String>,
}

impl Camotics {
    pub fn new(name: &str, tools: Vec<Tool>, workpiece: Bounds) -> Self {
        let mut tools_map = HashMap::new();

        for (index, tool) in tools.iter().enumerate() {
            let number = (index + 1) as u32;
            tools_map.insert(number, CamoticsTool::from_tool(*tool, number));
        }

        Self {
            name: name.to_string(),
            units: Units::Metric,
            resolution_mode: ResolutionMode::Manual,
            resolution: 0.5,
            tools: tools_map,
            workpiece: Workpiece {
                automatic: false,
                margin: 0.0,
                bounds: workpiece,
            },
            files: vec![format!("{}.ngc", name)],
        }
    }

    pub fn from_program(name: &str, program: Program) -> Self {
        let tools = program.tools();
        let workpiece = program.bounds();
        Self::new(name, tools, workpiece)
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use common_macros::hash_map;

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
            files: vec!["file.ngc".to_string()],
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
                    "file.ngc"
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_camotics_from_program() {
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
        });

        let camotics = Camotics::from_program("test-project", program.clone());

        assert_eq!(
            camotics,
            Camotics {
                name: "test-project".to_string(),
                units: Units::Metric,
                resolution_mode: ResolutionMode::Manual,
                resolution: 0.5,
                tools: hash_map! {1 => CamoticsTool::from_tool(tool, 1)},
                workpiece: Workpiece {
                    automatic: false,
                    margin: 0.0,
                    bounds: program.bounds()
                },
                files: vec!["test-project.ngc".to_string()]
            }
        );
    }
}
