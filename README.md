# cnccoder

cnccoder is a crate for writing cutting instructions and converting them to
[G-code](https://en.wikipedia.org/wiki/G-code) for use on 3 axis
[CNC machines](https://en.wikipedia.org/wiki/Numerical_control).

Rather than generating cuts from 3D models as FreeCAD and similar software,
it allows the user to precisely write the instructions for how the CNC machine
should cut and which [tools](tools/index.html) to use.

By providing several helper [cutting functions](cuts/index.html) and
[programs](programs/), the crate can then compile these higher level
instructions down to G-code.

The crate can also generate project files for [Camotics](https://camotics.org/)
that can be used to simulate the G-code so that it can be verified before ran
on an actual machine, reducing the risk of damaging the CNC machine and injury.

G-code does come in several flavors, but so far the project is only targeting
CNC machines using the [Grbl](https://github.com/gnea/grbl) controller.

## Usage example

Example of a simple planing program (from `examples/planing.rs`):
```
use anyhow::Result;
use cnccoder::prelude::*;

fn main() -> Result<()> {
    // Create a program with metric measurements where the tool can travel freely at 10 mm
    // height, and move to 50 mm height for manual tool change.
    let mut program = Program::new(Units::Metric, 10.0, 50.0);

    // Create a cylindrical tool
    let tool = Tool::cylindrical(
        Units::Metric, // Unit for tool measurements
        20.0, // Cutter length
        10.0, // Cutter diameter
        Direction::Clockwise, // Spindle rotation direction
        20000.0, // Spindle speed (rpm)
        5000.0, // Max feed rate/speed that the cutter will travel with (mm/min)
    );

    // Extend the program with the planing cuts
    program.extend(tool, |context| {
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

        Ok(())
    })?;

    // Write the G-code (for CNC) `planing.gcode` and Camotics project file
    // `planing.camotics` (for simulation) to disk using a resolution value
    // of 0.5 for the Camotics simulation.
    write_project("planing", program, 0.5)?;

    Ok(())
}
```

To run the G-code simulation in Camotics (if installed), simply run:
```bash
$ cargo run --example planing && camotics planing.camotics
```

The `cargo run --example planing` command generates the G-code file
`planing.gcode` and the Camotics project file `planing.camotics` in the
current directory.

The files can then be opened in Camotics by running `camotics planing.camotics`.

In this way you can easily simulate your projects.

## Whishlist for new features

* More ready to use programs for the `programs/` module, such as, boxes,
  reusable wood joints.
* WASM build and API, the current API is incompatible as it uses enum variants
  with values for tools and cuts. This is not yet supported by wasm-pack so an
  alternative API might be needed meanwhile.
* Support for font based text v carving.
* Support for creating a negative v carving cut for inlays.

## Contributing

You can help out in several ways:

1. Try out the crate
2. Report any issues, bugs, missing documentation or examples
3. Create issues with feedback on the ergonomy of the crate APIs
4. Extend the documentation or examples
5. Contribute code changes

Feedback on the ergonomics of this crate or its features/ lack there of might
be as valuable as code contributions.

### Code contributions

That being said, code contributions are more than welcome. Create a merge
request, with your new cuts, tools, programs or other improvements and create
a pull request.

Ensure that any new/changed publicly facing APIs have proper documentation
comments.

Once a pull request is ready for merge, also squash the commits into a single
commit per feature or fix.

The commit messages in this project should comply with the
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard
so that the [semver](https://semver.org/) versions can be automatically
calculated and the release changelog can be automatically generated.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or 
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
