use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{tools::*, types::*};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionMode {
    High,
    Low,
    Manual,
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Bounds {
    pub min: Vector3,
    pub max: Vector3,
}

impl Bounds {
    pub fn max() -> Self {
        Self {
            min: Vector3::max(),
            max: Vector3::min(),
        }
    }

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(x, y, z),
        }
    }

    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Workpiece {
    pub automatic: bool,
    pub margin: f64,
    pub bounds: Bounds,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CamoticsTool {
    pub units: Units,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    pub length: f64,
    pub diameter: f64,
    pub number: u32,
    pub shape: CamoticsToolShape,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn new(name: String, tools: Vec<Tool>, workpiece: Bounds) -> Self {
        let mut tools_map = HashMap::new();

        for (index, tool) in tools.iter().enumerate() {
            let number = (index + 1) as u32;
            tools_map.insert(number, CamoticsTool::from_tool(*tool, number));
        }

        Self {
            name: name.clone(),
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

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn persist_project(&self) -> Result<()> {
        let mut file = File::create(format!("{}.camotics", self.name))?;
        let json = self.to_json_string();
        let bytes = json.as_bytes();

        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

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
}
