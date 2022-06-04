use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{anyhow, Result};

use crate::cuts::*;
use crate::instructions::*;
use crate::tools::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub enum Operation {
    Cut(Cut),
    Empty(Empty),
    Comment(Comment),
    Message(Message),
}

impl Operation {
    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Cut(o) => o.bounds(),
            Self::Empty(_) => Bounds::default(),
            Self::Comment(_) => Bounds::default(),
            Self::Message(_) => Bounds::default(),
        }
    }

    pub fn to_instructions(&self, context: Context) -> Result<Vec<Instruction>> {
        match self {
            Self::Cut(o) => o.to_instructions(context),
            Self::Empty(_) => Ok(vec![Instruction::Empty(Empty {})]),
            Self::Comment(i) => Ok(vec![Instruction::Comment(i.clone())]),
            Self::Message(i) => Ok(vec![Instruction::Message(i.clone())]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    units: Units,
    tool: Tool,
    z_safe: f64,
    z_tool_change: f64,
    operations: Vec<Operation>,
}

impl Context {
    pub fn new(units: Units, tool: Tool, z_safe: f64, z_tool_change: f64) -> Self {
        Self {
            units,
            tool,
            z_safe,
            z_tool_change,
            operations: vec![],
        }
    }

    pub fn merge(&mut self, context: Context) -> Result<()> {
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

    pub fn append(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub fn append_cut(&mut self, cut: Cut) {
        self.append(Operation::Cut(cut));
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn z_safe(&self) -> f64 {
        self.z_safe
    }

    pub fn z_tool_change(&self) -> f64 {
        self.z_tool_change
    }

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

    pub fn to_instructions(&self) -> Result<Vec<Instruction>> {
        let mut instructions = vec![];

        for operation in &self.operations {
            instructions.append(&mut operation.to_instructions((*self).clone())?);
        }

        Ok(instructions)
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    z_safe: f64,
    z_tool_change: f64,
    units: Units,
    contexts: Arc<Mutex<HashMap<Tool, Arc<Mutex<Context>>>>>,
    tool_ordering: Arc<Mutex<HashMap<Tool, u32>>>,
}

impl Program {
    #[must_use]
    pub fn new(units: Units, z_safe: f64, z_tool_change: f64) -> Program {
        Program {
            z_safe,
            z_tool_change,
            units,
            contexts: Arc::new(Mutex::new(HashMap::new())),
            tool_ordering: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn z_safe(&self) -> f64 {
        self.z_safe
    }

    #[must_use]
    pub fn z_tool_change(&self) -> f64 {
        self.z_tool_change
    }

    #[must_use]
    pub fn tool_ordering(&self, tool: Tool) -> Option<u32> {
        if let Some(ordering) = self.tool_ordering.lock().unwrap().get(&tool) {
            return Some(*ordering);
        }

        None
    }

    pub fn set_tool_ordering(&self, tool: Tool, ordering: u32) {
        let mut tool_ordering = self.tool_ordering.lock().unwrap();

        for (it_tool, it_ordering) in tool_ordering.iter_mut() {
            if *it_tool != tool && *it_ordering >= ordering {
                *it_ordering += 1;
            }
        }

        tool_ordering.insert(tool, ordering);
    }

    fn create_context_if_missing_for_tool(&mut self, tool: Tool) {
        let mut contexts = self.contexts.lock().unwrap();
        if let Vacant(entry) = contexts.entry(tool) {
            let context = Context::new(self.units, tool, self.z_safe, self.z_tool_change);
            entry.insert(Arc::new(Mutex::new(context)));

            // Set tool ordering
            let mut tool_ordering = self.tool_ordering.lock().unwrap();
            let mut max_ordering = 0;
            for ordering in tool_ordering.values() {
                if *ordering > max_ordering {
                    max_ordering = *ordering;
                }
            }
            tool_ordering.insert(tool, max_ordering + 1);
        }
    }

    pub fn extend<Action>(&mut self, tool: Tool, action: Action) -> Result<()>
    where
        Action: Fn(&mut Context) -> Result<()>,
    {
        self.create_context_if_missing_for_tool(tool);
        let mut locked_contexts = self.contexts.lock().unwrap();
        let context = locked_contexts.get_mut(&tool).unwrap();
        let locked_context = &mut context.lock().unwrap();
        action(locked_context)
    }

    pub fn merge(&mut self, program: Program) -> Result<()> {
        if self.units != program.units {
            return Err(anyhow!("Failed to merge due to mismatching units"));
        }

        self.z_safe = self.z_safe.min(program.z_safe);
        self.z_tool_change = self.z_tool_change.min(program.z_tool_change);

        for tool in program.tools() {
            self.create_context_if_missing_for_tool(tool);
        }

        let program_contexts = program.contexts.lock().unwrap();
        let mut contexts = self.contexts.lock().unwrap();

        for tool in program.tools() {
            let program_context = program_contexts.get(&tool).unwrap().lock().unwrap();
            let context = &mut contexts.get_mut(&tool).unwrap().lock().unwrap();
            context.merge(program_context.clone())?;
        }

        Ok(())
    }

    #[must_use]
    pub fn tools(&self) -> Vec<Tool> {
        let mut tools = vec![];

        let tool_ordering = self.tool_ordering.lock().unwrap();
        let mut orderings: Vec<_> = tool_ordering.iter().collect();
        orderings.sort_by(|a, b| a.1.cmp(b.1));

        for (tool, _) in orderings {
            tools.push(*tool);
        }

        tools
    }

    #[must_use]
    pub fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::minmax();
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let context_bounds = context.lock().unwrap().bounds();
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

    #[must_use]
    pub fn to_instructions(&self) -> Result<Vec<Instruction>> {
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        let mut raw_instructions = vec![];

        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let locked_context = &mut context.lock().unwrap();
                let tool_number = self.tool_ordering(tool).unwrap();

                if tool_number > 1 {
                    raw_instructions.push(Instruction::Empty(Empty {}));
                }

                // Tool change
                raw_instructions.append(&mut vec![
                    Instruction::Comment(Comment {
                        text: format!("Tool change: {}", tool),
                    }),
                    match locked_context.units {
                        Units::Metric => Instruction::G21(G21 {}),
                        Units::Imperial => Instruction::G20(G20 {}),
                    },
                    Instruction::G0(G0 {
                        x: None,
                        y: None,
                        z: Some(locked_context.z_tool_change),
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
                raw_instructions.append(&mut locked_context.to_instructions()?);
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
        let raw_length = raw_instructions.len();
        let mut instructions = vec![];
        for (index, instruction) in (&raw_instructions).iter().enumerate() {
            if index < raw_length - 1 && instruction == &raw_instructions[index + 1] {
                continue;
            }

            instructions.push(instruction.clone());
        }

        Ok(instructions)
    }

    #[must_use]
    pub fn to_gcode(&self) -> Result<String> {
        Ok(self
            .to_instructions()?
            .iter()
            .map(|instruction| instruction.to_gcode())
            .collect::<Vec<String>>()
            .join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_program() {
        let program = Program::new(Units::Metric, 10.0, 50.0);
        assert_eq!(program.z_safe, 10.0);
        assert_eq!(program.z_tool_change, 50.0);
    }

    #[test]
    fn test_program_tools() -> Result<()> {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool1 = Tool::cylindrical(
            Units::Metric,
            50.0,
            4.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Metric,
            45.0,
            15.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        program.extend(tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        program.extend(tool2, |context| {
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

        program.set_tool_ordering(tool2, 0);

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
            5000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        program.extend(tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        program.extend(tool2, |context| {
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

        let instructions = program.to_instructions()?;

        let expected_output = vec![
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5000.0 }),
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
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5000.0 }),
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

        program.set_tool_ordering(tool2, 1);

        let instructions = program.to_instructions()?;

        let expected_output = vec![
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5000.0 }),
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
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5000.0 }),
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
            5000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        let mut program1 = Program::new(Units::Metric, 10.0, 40.0);

        program1.extend(tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        let mut program2 = Program::new(Units::Metric, 5.0, 50.0);

        program2.extend(tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(10.0, 10.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        program2.extend(tool2, |context| {
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

        program1.merge(program2)?;

        let instructions = program1.to_instructions()?;

        let expected_output = vec![
            Instruction::Comment(Comment { text: "Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400 mm/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 1 }),
            Instruction::S(S { x: 5000.0 }),
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
            Instruction::Comment(Comment { text: "Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000, feed_rate = 400\"/min".to_string() }),
            Instruction::G21(G21 {}),
            Instruction::G0(G0 { x: None, y: None, z: Some(50.0) }),
            Instruction::M5(M5 {}),
            Instruction::M6(M6 { t: 2 }),
            Instruction::S(S { x: 5000.0 }),
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
            5000.0,
            400.0,
        );

        let tool2 = Tool::conical(
            Units::Imperial,
            45.0,
            1.0,
            Direction::Clockwise,
            5000.0,
            400.0,
        );

        program.extend(tool1, |context| {
            context.append_cut(Cut::path(
                Vector3::new(0.0, 0.0, 3.0),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0,
            ));

            Ok(())
        })?;

        program.extend(tool2, |context| {
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

        program.set_tool_ordering(tool2, 1);

        let gcode = program.to_gcode()?;

        let expected_output = vec![
            ";(Tool change: Conical tool angle = 45°, diameter = 1\", length = 1.2071\", direction = clockwise, spindle_speed = 5000, feed_rate = 400\"/min)".to_string(),
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
            ";(Tool change: Cylindrical tool diameter = 4 mm, length = 50 mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400 mm/min)".to_string(),
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
