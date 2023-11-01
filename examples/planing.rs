use anyhow::Result;

use cnccoder::prelude::*;

fn main() -> Result<()> {
    // Create a program with metric measurements where the tool can travel freely at 10 mm
    // height, and move to 50 mm height for manual tool change.
    let mut program = Program::new(Units::Metric, 10.0, 50.0);

    // Create a cylindrical tool
    let tool = Tool::cylindrical(
        Units::Metric,        // Unit for tool measurements
        20.0,                 // Cutter length
        10.0,                 // Cutter diameter
        Direction::Clockwise, // Spindle rotation direction
        20000.0,              // Spindle speed (rpm)
        500.0,                // Max feed rate/speed that the cutter will travel with (mm/min)
    );

    // Get the tool context to extend the program
    let mut context = program.context(tool);

    // Append the planing cuts to the cylindrical tool context
    context.append_cut(Cut::plane(
        // Start at the x 0 mm, y 0 mm, z 3 mm coordinates
        Vector3::new(0.0, 0.0, 3.0),
        // Plane a 100 x 100 mm area
        Vector2::new(100.0, 100.0),
        // Plane down to 0 mm height (from 3 mm)
        0.0,
        // Cut at the most 1 mm per pass
        1.0,
    ));

    // Write the G-code (for CNC) `planing.gcode` and Camotics project file
    // `planing.camotics` (for simulation) to disk using a resolution value
    // of 0.5 for the Camotics simulation.
    write_project("planing", &program, 0.5)?;

    Ok(())
}
