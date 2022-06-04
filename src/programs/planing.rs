use anyhow::Result;

use crate::{Cut, Program, Tool, Units, Vector2, Vector3};

pub struct PlaningMeasurements {
    pub x_length: f64,
    pub y_length: f64,
    pub z_start: f64,
    pub z_end: f64,
    pub z_max_step: f64,
    pub units: Units,
}

impl Default for PlaningMeasurements {
    fn default() -> Self {
        let units = Units::default();

        Self {
            x_length: units.measurement_from_mm(10.0),
            y_length: units.measurement_from_mm(10.0),
            z_start: units.measurement_from_mm(5.0),
            z_end: units.measurement_from_mm(-0.1),
            z_max_step: units.measurement_from_mm(1.0),
            units,
        }
    }
}

pub fn planing(tool: Tool, measurements: PlaningMeasurements) -> Result<Program> {
    let mut program = Program::new(
        measurements.units,
        measurements.z_start + measurements.units.measurement_from_mm(2.0),
        measurements.z_start + measurements.units.measurement_from_mm(50.0),
    );

    program.extend(tool, |context| {
        context.append_cut(Cut::plane(
            Vector3::new(-tool.radius(), -tool.radius(), measurements.z_start),
            Vector2::new(
                measurements.x_length + tool.diameter(),
                measurements.y_length + tool.diameter(),
            ),
            measurements.z_end,
            measurements.z_max_step,
        ));

        Ok(())
    })?;

    Ok(program)
}
