use crate::prelude::*;

/// Measurements required by the planing program.
pub struct PlaningMeasurements {
    /// The length to plane on the x axis.
    pub x_length: f64,
    /// The length to plane on the y axis.
    pub y_length: f64,
    /// The height of where to start the planing on the z axis.
    pub z_start: f64,
    /// The depth of where to end the planing on the z axis.
    pub z_end: f64,
    /// The maximum depth to cut on the z axis on each pass.
    pub z_max_step: f64,
    /// The units used for the measurements.
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

/// A program for planing a surface down to a specific depth.
pub fn planing(tool: Tool, measurements: PlaningMeasurements) -> Program {
    let mut program = Program::new(
        measurements.units,
        measurements.z_start + measurements.units.measurement_from_mm(2.0),
        measurements.z_start + measurements.units.measurement_from_mm(50.0),
    );

    let mut context = program.context(tool);

    context.append_cut(Cut::plane(
        Vector3::new(-tool.radius(), -tool.radius(), measurements.z_start),
        Vector2::new(
            measurements.x_length + tool.diameter(),
            measurements.y_length + tool.diameter(),
        ),
        measurements.z_end,
        measurements.z_max_step,
    ));

    program
}
