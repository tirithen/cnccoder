//! The program module contains the highest level components. They are used to
//! structure the a CNC programs, store and order the cuts and tools.
//!
//! Programs are built by extending a program with various tool contexts.
//! Once a program is complete it can be converted to G-code with
//! the [.to_gcode()](struct.Program.html#method.to_gcode) method or written
//! to disk with the [write_project](../filesystem/fn.write_project.html)
//! function.
//!
//! Example:
//! ```
//! use anyhow::Result;
//! use cnccoder::prelude::*;
//!
//! fn main() -> Result<()> {
//!     let mut program = Program::new(
//!         Units::Metric,
//!         10.0,
//!         50.0,
//!     );
//!
//!     let tool = Tool::cylindrical(
//!         Units::Metric,
//!         20.0,
//!         10.0,
//!         Direction::Clockwise,
//!         20000.0,
//!         5_000.0
//!     );
//!
//!     let mut context = program.context(tool);
//!
//!     context.append_cut(Cut::plane(
//!         Vector3::new(0.0, 0.0, 3.0),
//!         Vector2::new(100.0, 100.0),
//!         0.0,
//!         1.0,
//!     ));
//!
//!     println!("G-code: {}", program.to_gcode()?);
//!
//!     write_project("planing", &program, 0.5)?;
//!
//!     Ok(())
//! }
//! ```

use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{anyhow, Result};

use crate::cuts::*;
use crate::instructions::*;
use crate::tools::*;
use crate::types::*;

/// A high level respresentation of a CNC program operation, Cut, Comment, Message, or Empty.
#[derive(Debug, Clone)]
pub enum Operation {
    /// A high level cut operation.
    Cut(Cut),
    /// An empty operation.
    Empty(Empty),
    /// A program comment.
    Comment(Comment),
    /// A program message.
    Message(Message),
}

impl Operation {
    /// The bounds of the operation.
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Cut(o) => o.bounds(),
            Self::Empty(_) => Bounds::default(),
            Self::Comment(_) => Bounds::default(),
            Self::Message(_) => Bounds::default(),
        }
    }

    /// Converts operation to G-code instructions.
    pub fn to_instructions(&self, context: InnerContext) -> Result<Vec<Instruction>> {
        match self {
            Self::Cut(o) => o.to_instructions(context),
            Self::Empty(_) => Ok(vec![Instruction::Empty(Empty {})]),
            Self::Comment(i) => Ok(vec![Instruction::Comment(i.clone())]),
            Self::Message(i) => Ok(vec![Instruction::Message(i.clone())]),
        }
    }
}

/// A program context that keeps the state data for operations paired with a specific tool.
/// The reason for grouping the operations per tool is to reduce the amound of tool
/// changes, which is expecially useful for CNC machines that needs manual tool changes.
///
/// This struct is mainly for internal use, most of the time you would use the ToolContext
/// struct instead.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct InnerContext {
    units: Units,
    tool: Tool,
    z_safe: f64,
    z_tool_change: f64,
    operations: Vec<Operation>,
}

impl InnerContext {
    /// Creates a new `Context` struct.
    pub fn new(units: Units, tool: &Tool, z_safe: f64, z_tool_change: f64) -> Self {
        Self {
            units,
            tool: *tool,
            z_safe,
            z_tool_change,
            operations: vec![],
        }
    }

    /// Applies operations from one context to this context.
    ///
    /// Returns error if tool or units are not the same in both contexts.
    pub fn merge(&mut self, context: InnerContext) -> Result<()> {
        if self.units != context.units {
            return Err(anyhow!("Failed to merge due to mismatching units"));
        }

        if self.tool != context.tool {
            return Err(anyhow!("Failed to merge due to mismatching tools"));
        }

        self.z_safe = context.z_safe;
        self.z_tool_change = context.z_tool_change;

        for operation in context.operations {
            self.operations.push(operation);
        }

        Ok(())
    }

    /// Appends an operation to the context.
    pub fn append(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    /// Appends a cut operation to the context.
    pub fn append_cut(&mut self, cut: Cut) {
        self.append(Operation::Cut(cut));
    }

    /// Returns the units used by the context.
    pub fn units(&self) -> Units {
        self.units
    }

    /// Returns the tool used by the context.
    pub fn tool(&self) -> Tool {
        self.tool
    }

    /// Returns the z safe value set for this context.
    ///
    /// The value indicates the z height where the machine tool can safely travel
    /// in the x and y axis without colliding with the workpiece.
    pub fn z_safe(&self) -> f64 {
        self.z_safe
    }

    /// Returns the z height position used for manual tool change.
    pub fn z_tool_change(&self) -> f64 {
        self.z_tool_change
    }

    /// Returns the bounds for the context
    pub fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::minmax();

        for operation in self.operations.iter() {
            let operation_bounds = operation.bounds();
            bounds.min.x = if bounds.min.x > operation_bounds.min.x {
                operation_bounds.min.x
            } else {
                bounds.min.x
            };
            bounds.min.y = if bounds.min.y > operation_bounds.min.y {
                operation_bounds.min.y
            } else {
                bounds.min.y
            };
            bounds.min.z = if bounds.min.z > operation_bounds.min.z {
                operation_bounds.min.z
            } else {
                bounds.min.z
            };
            bounds.max.x = if bounds.max.x < operation_bounds.max.x {
                operation_bounds.max.x
            } else {
                bounds.max.x
            };
            bounds.max.y = if bounds.max.y < operation_bounds.max.y {
                operation_bounds.max.y
            } else {
                bounds.max.y
            };
            bounds.max.z = if bounds.max.z < operation_bounds.max.z {
                operation_bounds.max.z
            } else {
                bounds.max.z
            };
        }

        bounds
    }

    /// Returns all operations for this context.
    pub fn operations(&self) -> Vec<Operation> {
        self.operations.clone()
    }

    /// Converts context to G-code instructions.
    pub fn to_instructions(&self) -> Result<Vec<Instruction>> {
        let mut instructions = vec![];

        for operation in &self.operations {
            instructions.append(&mut operation.to_instructions((*self).clone())?);
        }

        Ok(instructions)
    }
}

/// A program tool context that updates the state data for operations paired with a specific
/// tool. The reason for grouping the operations per tool is to reduce the amound of tool
/// changes, which is expecially useful for CNC machines that needs manual tool changes.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    tool: Tool,
    program: Arc<Mutex<&'a Program>>,
}

impl<'a> Context<'a> {
    /// Applies operations from one context to this context.
    ///
    /// Returns error if tool or units are not the same in both contexts.
    pub fn merge(&mut self, context: Context) -> Result<()> {
        let program = self.program.lock().unwrap();

        let mut binding = program.contexts.lock().unwrap();
        let program_context = binding.get_mut(&self.tool).unwrap();

        let binding = context.program.lock().unwrap().contexts.lock().unwrap();
        let merge_context = binding.get(&context.tool()).unwrap();

        program_context.merge(merge_context.clone())
    }

    /// Appends an operation to the context.
    pub fn append(&mut self, operation: Operation) {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.append(operation);
    }

    /// Appends a cut operation to the context.
    pub fn append_cut(&mut self, cut: Cut) {
        self.append(Operation::Cut(cut));
    }

    /// Returns the units used by the context.
    pub fn units(&self) -> Units {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.units()
    }

    /// Returns the tool used by the context.
    pub fn tool(&self) -> Tool {
        self.tool
    }

    /// Returns the z safe value set for this context.
    ///
    /// The value indicates the z height where the machine tool can safely travel
    /// in the x and y axis without colliding with the workpiece.
    pub fn z_safe(&self) -> f64 {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.z_safe()
    }

    /// Returns the z height position used for manual tool change.
    pub fn z_tool_change(&self) -> f64 {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.z_tool_change()
    }

    /// Returns the bounds for this context.
    pub fn bounds(&self) -> Bounds {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.bounds()
    }

    /// Returns all operations for this context.
    pub fn operations(&self) -> Vec<Operation> {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.operations()
    }

    /// Converts context to G-code instructions.
    pub fn to_instructions(&self) -> Result<Vec<Instruction>> {
        let program = self.program.lock().unwrap();
        let mut binding = program.contexts.lock().unwrap();
        let context = binding.get_mut(&self.tool).unwrap();
        context.to_instructions()
    }
}

/// A program that stores information about all structs and tools used in a project. Several programs can
/// also be merged into a single one.
#[derive(Debug, Clone)]
pub struct Program {
    z_safe: f64,
    z_tool_change: f64,
    units: Units,
    contexts: Arc<Mutex<HashMap<Tool, InnerContext>>>,
    tool_ordering: Arc<Mutex<ToolOrdering>>,
}

impl Program {
    /// Creates a new `Program` struct.
    #[must_use]
    pub fn new(units: Units, z_safe: f64, z_tool_change: f64) -> Self {
        Self {
            z_safe,
            z_tool_change,
            units,
            contexts: Arc::new(Mutex::new(HashMap::new())),
            tool_ordering: Arc::new(Mutex::new(ToolOrdering::default())),
        }
    }

    /// Creates a new empty `Program` with the same same settings as the supplied one.
    #[must_use]
    pub fn new_empty_from(program: &Self) -> Self {
        Self {
            z_safe: program.z_safe,
            z_tool_change: program.z_tool_change,
            units: program.units,
            contexts: Arc::new(Mutex::new(HashMap::new())),
            tool_ordering: Arc::new(Mutex::new(ToolOrdering::default())),
        }
    }

    /// Returns the z safe value set for this context.
    ///
    /// The value indicates the z height where the machine tool can safely travel
    /// in the x and y axis without colliding with the workpiece.
    #[must_use]
    pub fn z_safe(&self) -> f64 {
        self.z_safe
    }

    /// Returns the z height position used for manual tool change.
    #[must_use]
    pub fn z_tool_change(&self) -> f64 {
        self.z_tool_change
    }

    /// Returns the tools position in a program, this number will then be used in the G-code T commands
    /// (T1 is the first tool, T2 is the second tool and so on).
    #[must_use]
    pub fn tool_ordering(&self, tool: &Tool) -> Option<u8> {
        let tool_ordering = self.tool_ordering.lock().unwrap();
        tool_ordering.ordering(tool)
    }

    /// Allows setting the positional order for a tool, this will also automatically increment the position
    /// of any tools that comes after the newly repositioned tool, resolving any ordering conflicts.
    pub fn set_tool_ordering(&self, tool: &Tool, ordering: u8) {
        let mut tool_ordering = self.tool_ordering.lock().unwrap();
        tool_ordering.set_ordering(tool, ordering);
    }

    fn create_context_if_missing_for_tool(&mut self, tool: &Tool) {
        let mut contexts = self.contexts.lock().unwrap();
        if let Vacant(entry) = contexts.entry(*tool) {
            let context = InnerContext::new(self.units, tool, self.z_safe, self.z_tool_change);
            entry.insert(context);

            let mut tool_ordering = self.tool_ordering.lock().unwrap();
            tool_ordering.auto_ordering(tool);
        }
    }

    /// This is the main way of adding cuts to a program.
    /// It returns a new tool context that can be used to extend the program.
    ///
    /// An example for adding cuts to a program:
    /// ```
    /// use cnccoder::prelude::*;
    ///
    /// let mut program = Program::default();
    /// let tool = Tool::default();
    ///
    /// // Extend the program with new cuts
    /// let mut context = program.context(tool);
    ///
    /// // Append the planing cuts to the cylindrical tool context
    /// context.append_cut(Cut::plane(
    ///     // Start at the x 0 mm, y 0 mm, z 3 mm coordinates
    ///     Vector3::new(0.0, 0.0, 3.0),
    ///     // Plane a 100 x 100 mm area
    ///     Vector2::new(100.0, 100.0),
    ///     // Plane down to 0 mm height (from 3 mm)
    ///     0.0,
    ///     // Cut at the most 1 mm per pass
    ///     1.0,
    /// ));
    /// ```
    pub fn context(&mut self, tool: Tool) -> Context {
        self.create_context_if_missing_for_tool(&tool);
        Context {
            tool,
            program: Arc::new(Mutex::new(self)),
        }
    }

    /// This is the main way of adding cuts to a program.
    /// It opens a new context for a tool where the program can be extended.
    ///
    /// An example for adding cuts to a program:
    /// ```
    /// use anyhow::Result;
    /// use cnccoder::prelude::*;
    ///
    /// fn main() -> Result<()> {
    ///     let mut program = Program::default();
    ///     let tool = Tool::default();
    ///
    ///     // Extend the program with new cuts
    ///     program.extend(&tool, |context| {
    ///         // Append the planing cuts to the cylindrical tool context
    ///         context.append_cut(Cut::plane(
    ///             // Start at the x 0 mm, y 0 mm, z 3 mm coordinates
    ///             Vector3::new(0.0, 0.0, 3.0),
    ///             // Plane a 100 x 100 mm area
    ///             Vector2::new(100.0, 100.0),
    ///             // Plane down to 0 mm height (from 3 mm)
    ///             0.0,
    ///             // Cut at the most 1 mm per pass
    ///             1.0,
    ///         ));
    ///
    ///         Ok(())
    ///     })?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "Replaced with the .context method that does not require operations to be added via closures."
    )]
    pub fn extend<Action>(&mut self, tool: &Tool, action: Action) -> Result<()>
    where
        Action: Fn(&mut InnerContext) -> Result<()>,
    {
        self.create_context_if_missing_for_tool(tool);
        let mut locked_contexts = self.contexts.lock().unwrap();
        let context = locked_contexts.get_mut(tool).unwrap();
        action(context)
    }

    /// Merges another program into this program.
    ///
    /// Returns error if tool or units are not the same in both programs.
    pub fn merge(&mut self, program: &Program) -> Result<()> {
        if self.units != program.units {
            return Err(anyhow!("Failed to merge due to mismatching units"));
        }

        self.z_safe = self.z_safe.min(program.z_safe);
        self.z_tool_change = self.z_tool_change.min(program.z_tool_change);

        for tool in program.tools() {
            self.create_context_if_missing_for_tool(&tool);
        }

        let program_contexts = program.contexts.lock().unwrap();
        let mut contexts = self.contexts.lock().unwrap();

        for tool in program.tools() {
            let program_context = program_contexts.get(&tool).unwrap();
            let context = &mut contexts.get_mut(&tool).unwrap();
            context.merge(program_context.clone())?;
        }

        Ok(())
    }

    /// Returns an ordered vec with all tools used by a program.
    #[must_use]
    pub fn tools(&self) -> Vec<Tool> {
        let tool_ordering = self.tool_ordering.lock().unwrap();
        tool_ordering.tools_ordered()
    }

    /// Returns the bounds of the program.
    #[must_use]
    pub fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::minmax();
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let context_bounds = context.bounds();
                bounds.min.x = if bounds.min.x > context_bounds.min.x {
                    context_bounds.min.x
                } else {
                    bounds.min.x
                };
                bounds.min.y = if bounds.min.y > context_bounds.min.y {
                    context_bounds.min.y
                } else {
                    bounds.min.y
                };
                bounds.min.z = if bounds.min.z > context_bounds.min.z {
                    context_bounds.min.z
                } else {
                    bounds.min.z
                };
                bounds.max.x = if bounds.max.x < context_bounds.max.x {
                    context_bounds.max.x
                } else {
                    bounds.max.x
                };
                bounds.max.y = if bounds.max.y < context_bounds.max.y {
                    context_bounds.max.y
                } else {
                    bounds.max.y
                };
                bounds.max.z = if bounds.max.z < context_bounds.max.z {
                    context_bounds.max.z
                } else {
                    bounds.max.z
                };
            }
        }

        bounds
    }

    /// Converts a program to G-code instructions
    pub fn to_instructions(&self) -> Result<Vec<Instruction>> {
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        let mut raw_instructions = vec![Instruction::G17(G17 {})];

        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let tool_number = self.tool_ordering(&tool).unwrap();

                raw_instructions.push(Instruction::Empty(Empty {}));

                // Tool change
                raw_instructions.append(&mut vec![
                    Instruction::Comment(Comment {
                        text: format!("Tool change: {}", tool),
                    }),
                    match context.units {
                        Units::Metric => Instruction::G21(G21 {}),
                        Units::Imperial => Instruction::G20(G20 {}),
                    },
                    Instruction::G0(G0 {
                        x: None,
                        y: None,
                        z: Some(context.z_tool_change),
                    }),
                    Instruction::M5(M5 {}),
                    Instruction::M6(M6 { t: tool_number }),
                    Instruction::S(S {
                        x: tool.spindle_speed(),
                    }),
                    if tool.direction() == Direction::Clockwise {
                        Instruction::M3(M3 {})
                    } else {
                        Instruction::M4(M4 {})
                    },
                ]);

                // Add tool instructions
                raw_instructions.append(&mut context.to_instructions()?);
            }
        }

        // End program
        raw_instructions.push(Instruction::G0(G0 {
            x: None,
            y: None,
            z: Some(self.z_tool_change),
        }));
        raw_instructions.push(Instruction::Empty(Empty {}));
        raw_instructions.push(Instruction::M2(M2 {}));

        // Trim duplicated instructions
        let mut workplane = Instruction::Empty(Empty {});
        let raw_length = raw_instructions.len();
        let mut instructions = vec![];
        for (index, instruction) in raw_instructions.iter().enumerate() {
            if *instruction == Instruction::G17(G17 {})
                || *instruction == Instruction::G18(G18 {})
                || *instruction == Instruction::G19(G19 {})
            {
                if *instruction == workplane {
                    continue;
                } else {
                    workplane = instruction.clone();
                }
            }

            if index < raw_length - 1 && instruction == &raw_instructions[index + 1] {
                continue;
            }

            instructions.push(instruction.clone());
        }

        Ok(instructions)
    }

    /// Converts program to G-code
    pub fn to_gcode(&self) -> Result<String> {
        Ok(self
            .to_instructions()?
            .iter()
            .map(|instruction| instruction.to_gcode())
            .collect::<Vec<String>>()
            .join("\n"))
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            z_safe: 50.0,
            z_tool_change: 100.0,
            units: Units::default(),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            tool_ordering: Arc::new(Mutex::new(ToolOrdering::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_new() {
        let program = Program::new(Units::Metric, 10.0, 50.0);
        assert_eq!(program.z_safe, 10.0);
        assert_eq!(program.z_tool_change, 50.0);
    }

    #[test]
    fn test_program_empty() {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let mut context = program.context(tool);
        context.append_cut(Cut::drill(Vector3::default(), -1.0));

        assert_eq!(program.tools().len(), 1);
        assert_eq!(program.to_instructions().unwrap(), vec![
            Instruction::G17(G17 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Drill hole at: x = 0, y = 0".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(0.0), y: Some(0.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(-1.0), f: Some(400.0) }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ]);

        let other_program = Program::new_empty_from(&program);

        assert_eq!(other_program.z_safe, 10.0);
        assert_eq!(other_program.z_tool_change, 50.0);
        assert_eq!(other_program.tools().len(), 0);
        assert_eq!(
            other_program.to_instructions().unwrap(),
            vec![
                Instruction::G17(G17 {}),
                Instruction::G0(G0 {
                    x: None,
                    y: None,
                    z: Some(50.0)
                }),
                Instruction::Empty(Empty {}),
                Instruction::M2(M2 {}),
            ]
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_program_extend() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Metric,
            45.0,
            15.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        program.extend(&tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        program.extend(&tool2, |context| {
            context.append_cut(Cut::path(
                Vector3::new(5.0, 10.0, 3.0),
                vec![Segment::line(
                    Vector2::new(5.0, 10.0),
                    Vector2::new(15.0, 10.0),
                )],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        let tools = program.tools();
        assert_eq!(tools, vec![tool1, tool2]);

        program.set_tool_ordering(&tool2, 0);

        let tools = program.tools();
        assert_eq!(tools, vec![tool2, tool1]);

        Ok(())
    }

    #[test]
    fn test_program_tools() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Metric,
            45.0,
            15.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let mut tool1_context = program.context(tool1);
        tool1_context.append_cut(Cut::path(
            Vector3::new(0.0, 0.0, 3.0),
            vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
            -0.1,
            1.0,
        ));

        let mut tool2_context = program.context(tool2);
        tool2_context.append_cut(Cut::path(
            Vector3::new(5.0, 10.0, 3.0),
            vec![Segment::line(
                Vector2::new(5.0, 10.0),
                Vector2::new(15.0, 10.0),
            )],
            -0.1,
            1.0,
        ));

        let tools = program.tools();
        assert_eq!(tools, vec![tool1, tool2]);

        program.set_tool_ordering(&tool2, 0);

        let tools = program.tools();
        assert_eq!(tools, vec![tool2, tool1]);

        Ok(())
    }

    #[test]
    fn test_program_to_instructions() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let mut tool1_context = program.context(tool1);
        tool1_context.append_cut(Cut::path(
            Vector3::new(0.0, 0.0, 3.0),
            vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
            -0.1,
            1.0,
        ));

        let mut tool2_context = program.context(tool2);
        tool2_context.append_cut(Cut::path(
            Vector3::new(5.0, 10.0, 3.0),
            vec![Segment::line(
                Vector2::new(5.0, 10.0),
                Vector2::new(15.0, 10.0),
            )],
            -0.1,
            1.0,
        ));

        let instructions = program.to_instructions()?;

        let expected_output = vec![
            Instruction::G17(G17 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 0, y = 0".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(0.0), y: Some(0.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 5, y = 10".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(10.0), y: Some(20.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ];

        assert_eq!(instructions, expected_output);

        program.set_tool_ordering(&tool2, 1);

        let instructions = program.to_instructions()?;

        let expected_output = vec![
            Instruction::G17(G17 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 5, y = 10".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(10.0), y: Some(20.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 0, y = 0".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(0.0), y: Some(0.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ];

        assert_eq!(instructions, expected_output);

        Ok(())
    }

    #[test]
    fn test_merge_programs() -> Result<()> {
        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let mut program1 = Program::new(Units::Metric, 10.0, 40.0);

        let mut program1_tool1_context = program1.context(tool1);
        program1_tool1_context.append_cut(Cut::path(
            Vector3::new(0.0, 0.0, 3.0),
            vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
            -0.1,
            1.0,
        ));

        let mut program2 = Program::new(Units::Metric, 5.0, 50.0);

        let mut program2_tool1_context = program2.context(tool1);
        program2_tool1_context.append_cut(Cut::path(
            Vector3::new(10.0, 10.0, 3.0),
            vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
            -0.1,
            1.0,
        ));

        let mut program2_tool2_context = program2.context(tool2);
        program2_tool2_context.append_cut(Cut::path(
            Vector3::new(5.0, 10.0, 3.0),
            vec![Segment::line(
                Vector2::new(5.0, 10.0),
                Vector2::new(15.0, 10.0),
            )],
            -0.1,
            1.0,
        ));

        program1.merge(&program2)?;

        let instructions = program1.to_instructions()?;

        let expected_output = vec![
            Instruction::G17(G17 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 0, y = 0".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::G0(G0 { x: Some(0.0), y: Some(0.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 10, y = 10".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::G0(G0 { x: Some(10.0), y: Some(10.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(10.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(15.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(10.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(15.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(10.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(15.0), y: Some(20.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(10.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(15.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5_000.0 }),
            Instruction::M3(M3 {}),
            Instruction::Empty(Empty {}),
            Instruction::Comment(Comment { text: "Cut path at: x = 5, y = 10".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::G0(G0 { x: Some(10.0), y: Some(20.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(3.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(3.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(2.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(1.0), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(0.0), f: None }),
            Instruction::G1(G1 { x: Some(10.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(20.0), y: Some(20.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(5.0) }),
            Instruction::G0(G0 { x: None, y: None, z: Some(40.0) }),
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ];

        assert_eq!(instructions, expected_output);

        Ok(())
    }

    #[test]
    fn test_program_to_gcode() -> Result<()> {
        let mut program = Program::new(Units::Imperial, 10.0, 50.0);

        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5_000.0,
            400.0,
        );

        let mut tool1_context = program.context(tool1);
        tool1_context.append_cut(Cut::path(
            Vector3::new(0.0, 0.0, 3.0),
            vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
            -0.1,
            1.0,
        ));

        let mut tool2_context = program.context(tool2);
        tool2_context.append_cut(Cut::path(
            Vector3::new(5.0, 10.0, 3.0),
            vec![Segment::line(
                Vector2::new(5.0, 10.0),
                Vector2::new(15.0, 10.0),
            )],
            -0.1,
            1.0,
        ));

        program.set_tool_ordering(&tool2, 0);

        let gcode = program.to_gcode()?;

        let expected_output = vec![
            "G17".to_string(),
            "".to_string(),
            ";(Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400\"/min)".to_string(),
            "G20".to_string(),
            "G0 Z50".to_string(),
            "M5".to_string(),
            "T1 M6".to_string(),
            "S5000".to_string(),
            "M3".to_string(),
            "".to_string(),
            ";(Cut path at: x = 5, y = 10)".to_string(),
            "G0 Z10".to_string(),
            "G0 X10 Y20".to_string(),
            "G1 Z3 F400".to_string(),
            "G1 X10 Y20 Z3".to_string(),
            "G1 X20 Y20 Z2".to_string(),
            "G1 X10 Y20 Z2".to_string(),
            "G1 X20 Y20 Z1".to_string(),
            "G1 X10 Y20 Z1".to_string(),
            "G1 X20 Y20 Z0".to_string(),
            "G1 X10 Y20 Z-0.1".to_string(),
            "G1 X20 Y20 Z-0.1".to_string(),
            "G0 Z10".to_string(),
            "".to_string(),
            ";(Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000 rpm, feed_rate = 400 mm/min)".to_string(),
            "G20".to_string(),
            "G0 Z50".to_string(),
            "M5".to_string(),
            "T2 M6".to_string(),
            "S5000".to_string(),
            "M3".to_string(),
            "".to_string(),
            ";(Cut path at: x = 0, y = 0)".to_string(),
            "G0 Z10".to_string(),
            "G0 X0 Y0".to_string(),
            "G1 Z3 F400".to_string(),
            "G1 X0 Y0 Z3".to_string(),
            "G1 X5 Y10 Z2".to_string(),
            "G1 X0 Y0 Z2".to_string(),
            "G1 X5 Y10 Z1".to_string(),
            "G1 X0 Y0 Z1".to_string(),
            "G1 X5 Y10 Z0".to_string(),
            "G1 X0 Y0 Z-0.1".to_string(),
            "G1 X5 Y10 Z-0.1".to_string(),
            "G0 Z10".to_string(),
            "G0 Z50".to_string(),
            "".to_string(),
            "M2".to_string(),
        ].join("\n");

        assert_eq!(gcode, expected_output);

        Ok(())
    }

    #[test]
    fn test_program_bounds() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5_000.0,
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

        let bounds = program.bounds();

        assert_eq!(
            bounds,
            Bounds {
                min: Vector3::new(-28.0, -30.0, -0.1),
                max: Vector3::new(67.0, 102.0, 3.0),
            }
        );

        Ok(())
    }
}
