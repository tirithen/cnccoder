#![warn(missing_docs)]

//! cnccoder is a crate for writing cutting instructions and converting them to
//! [G-code](https://en.wikipedia.org/wiki/G-code) for use on 3 axis
//! [CNC machines](https://en.wikipedia.org/wiki/Numerical_control).
//!
//! Rather than generating cuts from 3D models as FreeCAD and similar software,
//! it allows the user to precisely write the instructions for how the CNC machine
//! should cut and which [tools](tools/index.html) to use.
//!
//! By providing several helper [cutting functions](cuts/index.html) and
//! [programs](programs/), the crate can then compile these higher level
//! instructions down to G-code.
//!
//! The crate can also generate project files for [Camotics](https://camotics.org/)
//! that can be used to simulate the G-code so that it can be verified before ran
//! on an actual machine, reducing the risk of damaging the CNC machine and injury.
//!
//! G-code does come in several flavors, but so far the project is only targeting
//! CNC machines using the [Grbl](https://github.com/gnea/grbl) controller.
//!
//! Example of a simple planing program (from `examples/planing.rs`):
//! ```
//! use anyhow::Result;
//! use cnccoder::prelude::*;
//!
//! fn main() -> Result<()> {
//!     // Create a program with metric measurements where the tool can travel freely at 10 mm
//!     // height, and move to 50 mm height for manual tool change.
//!     let mut program = Program::new(Units::Metric, 10.0, 50.0);
//!
//!     // Create a cylindrical tool
//!     let tool = Tool::cylindrical(
//!         Units::Metric, // Unit for tool measurements
//!         20.0, // Cutter length
//!         10.0, // Cutter diameter
//!         Direction::Clockwise, // Spindle rotation direction
//!         20000.0, // Spindle speed (rpm)
//!         5000.0, // Max feed rate/speed that the cutter will travel with (mm/min)
//!     );
//!
//!     // Extend the program with the planing cuts
//!     program.extend(tool, |context| {
//!         // Append the planing cuts to the cylindrical tool context
//!         context.append_cut(Cut::plane(
//!             // Start at the x 0 mm, y 0 mm, z 3 mm coordinates
//!             Vector3::new(0.0, 0.0, 3.0),
//!             // Plane a 100 x 100 mm area
//!             Vector2::new(100.0, 100.0),
//!             // Plane down to 0 mm height (from 3 mm)
//!             0.0,
//!             // Cut at the most 1 mm per pass
//!             1.0,
//!         ));
//!
//!         Ok(())
//!     })?;
//!
//!     // Write the G-code (for CNC) `planing.gcode` and Camotics project file
//!     // `planing.camotics` (for simulation) to disk using a resolution value
//!     // of 0.5 for the Camotics simulation.
//!     write_project("planing", program, 0.5)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! To run the G-code simulation in Camotics (if installed), simply run:
//! ```bash
//! $ cargo run --example planing && camotics planing.camotics
//! ```
//!
//! The `cargo run --example planing` command generates the G-code file
//! `planing.gcode` and the Camotics project file `planing.camotics` in the
//! current directory.
//!
//! The files can then be opened in Camotics by running `camotics planing.camotics`.
//!
//! In this way you can easily simulate your projects.

#[cfg(feature = "filesystem")]
pub mod camotics;
pub mod cuts;
pub mod instructions;
pub mod program;
pub mod programs;
pub mod tools;
pub mod types;
pub mod utils;

#[cfg(feature = "filesystem")]
pub mod filesystem;

/// A prelude module to simplify imports in projects using this crate.
///
/// All public components can be imported in a project with:
/// ```
/// use cnccoder::prelude::*;
///
/// let program = Program::new(Units::Metric, 5.0, 50.0);
/// // The rest of your CNC program...
/// ```
pub mod prelude {
    #[cfg(feature = "filesystem")]
    #[doc(hidden)]
    pub use crate::camotics::*;
    #[doc(hidden)]
    pub use crate::cuts::*;
    #[cfg(feature = "filesystem")]
    #[doc(hidden)]
    pub use crate::filesystem::*;
    #[doc(hidden)]
    pub use crate::instructions::*;
    #[doc(hidden)]
    pub use crate::program::*;
    #[doc(hidden)]
    pub use crate::programs::*;
    #[doc(hidden)]
    pub use crate::tools::*;
    #[doc(hidden)]
    pub use crate::types::*;
    #[doc(hidden)]
    pub use crate::utils::*;
}
