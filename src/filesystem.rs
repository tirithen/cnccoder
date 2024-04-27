//! Provides helpers for writing G-code and project files to disk.

use std::{fs::File, io::Write};

use anyhow::Result;

use crate::{camotics::*, program::*};

/// Writes .gcode and .camotics files from a program to disk.
///
/// Example for planing a surface area to a specific height (z axis is vertical):
/// ```
/// use anyhow::Result;
/// use cnccoder::prelude::*;
///
/// fn main() -> Result<()> {
///     let mut program = Program::new(
///         Units::Metric,
///         10.0,
///         50.0,
///     );
///     program.set_name("planing");
///
///     let tool = Tool::cylindrical(
///         Units::Metric,
///         20.0,
///         10.0,
///         Direction::Clockwise,
///         20000.0,
///         5000.0
///     );
///
///     let mut context = program.context(tool);
///
///     context.append_cut(Cut::plane(
///         Vector3::new(0.0, 0.0, 3.0),
///         Vector2::new(100.0, 100.0),
///         0.0,
///         1.0,
///     ));
///
///     write_project(&program, 0.5)?;
///
///     Ok(())
/// }
/// ```
pub fn write_project(program: &Program, camotics_resolution: f64) -> Result<()> {
    let name = program.name();
    let camotics = Camotics::from_program(name, program, camotics_resolution);
    let gcode = program.to_gcode()?;

    let mut camotics_file = File::create(format!("{}.camotics", name))?;
    camotics_file.write_all(camotics.to_json_string().as_bytes())?;
    camotics_file.sync_all()?;

    let mut gcode_file = File::create(format!("{}.gcode", name))?;
    gcode_file.write_all(gcode.as_bytes())?;
    gcode_file.sync_all()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{read_to_string, remove_file};

    use anyhow::Result;
    use serde_json::Value;

    use crate::{cuts::*, tools::*, types::*};

    use super::*;

    #[test]
    fn test_camotics_from_program() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);
        program.set_name("test-temp");

        let tool = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        let mut context = program.context(tool);

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

        write_project(&program, 0.5)?;

        let camotics: Value = serde_json::from_str(&read_to_string("test-temp.camotics")?)?;
        remove_file("test-temp.camotics")?;

        let expected_camotics_output: Value = serde_json::from_str(
            r#"{
            "units": "metric",
            "resolution-mode": "manual",
            "resolution": 0.5,
            "tools": {
                "1": {
                "units": "metric",
                "length": 50.0,
                "diameter": 4.0,
                "number": 1,
                "shape": "cylindrical"
                }
            },
            "workpiece": {
                "automatic": false,
                "margin": 0.0,
                "bounds": {
                "min": [
                    -28.0,
                    -30.0,
                    -0.1
                ],
                "max": [
                    67.0,
                    102.0,
                    3.0
                ]
                }
            },
            "files": [
                "test-temp.gcode"
            ]
        }"#,
        )?;

        assert_eq!(camotics, expected_camotics_output);

        let gcode = read_to_string("test-temp.gcode")?;
        remove_file("test-temp.gcode")?;

        assert_eq!(gcode, r#"G17

;(Tool change: type = Cylindrical, diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min)
G21
G0 Z50
M5
T1 M6
S5000
M3

;(Cut path at: x = 0, y = 0)
G0 Z10
G0 X0 Y0
G1 Z3 F400
G1 X0 Y0 Z3
G1 X-28 Y-30 Z2
G1 X0 Y0 Z2
G1 X-28 Y-30 Z1
G1 X0 Y0 Z1
G1 X-28 Y-30 Z0
G1 X0 Y0 Z-0.1
G1 X-28 Y-30 Z-0.1
G0 Z10

;(Cut path at: x = 0, y = 0)
G0 Z10
G0 X23 Y12
G1 Z3 F400
G1 X23 Y12 Z3
G1 X5 Y10 Z2.95
G1 X67 Y102 Z2.451
G1 X23 Y12 Z2
G1 X23 Y12 Z2
G1 X5 Y10 Z1.95
G1 X67 Y102 Z1.451
G1 X23 Y12 Z1
G1 X5 Y10 Z0.95
G1 X67 Y102 Z0.451
G1 X23 Y12 Z-0
G1 X23 Y12 Z-0.1
G1 X5 Y10 Z-0.1
G1 X67 Y102 Z-0.1
G1 X23 Y12 Z-0.1
G0 Z10
G0 Z50

M2"#.to_string());

        Ok(())
    }
}
