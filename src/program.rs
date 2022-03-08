use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cuts::*;
use crate::instructions::*;
use crate::tools::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub enum Operation {
    Cut(Cut),
}

impl Operation {
    pub fn to_instructions(&self, context: Context) -> Vec<Instruction> {
        match self {
            Self::Cut(cut) => cut.to_instructions(context),
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

    pub fn to_instructions(&self) -> Vec<Instruction> {
        let mut instructions = vec![];

        for operation in &self.operations {
            instructions.append(&mut operation.to_instructions((*self).clone()));
        }

        instructions
    }
}

pub struct Program {
    z_safe: f64,
    z_tool_change: f64,
    units: Units,
    contexts: Arc<Mutex<HashMap<Tool, Arc<Mutex<Context>>>>>,
    tool_ordering: Arc<Mutex<HashMap<Tool, u32>>>,
}

impl Program {
    pub fn new(units: Units, z_safe: f64, z_tool_change: f64) -> Program {
        Program {
            z_safe,
            z_tool_change,
            units,
            contexts: Arc::new(Mutex::new(HashMap::new())),
            tool_ordering: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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

    pub fn extend<Action>(&mut self, tool: Tool, action: Action)
    where
        Action: Fn(&mut Context),
    {
        self.create_context_if_missing_for_tool(tool);
        let mut locked_contexts = self.contexts.lock().unwrap();
        let context = locked_contexts.get_mut(&tool).unwrap();
        let locked_context = &mut context.lock().unwrap();
        action(locked_context);
    }

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

    pub fn to_instructions(&self) -> Vec<Instruction> {
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        let mut raw_instructions = vec![
            match self.units {
                Units::Metric => Instruction::G21(G21 {}),
                Units::Imperial => Instruction::G20(G20 {}),
            },
            Instruction::Empty(Empty {}),
        ];

        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let locked_context = &mut context.lock().unwrap();

                // Tool change
                raw_instructions.append(&mut vec![
                    Instruction::Empty(Empty {}),
                    Instruction::Message(Message { text: format!("Tool change: {}", tool)}),
                    Instruction::G0(G0 {
                        x: None,
                        y: None,
                        z: Some(locked_context.z_tool_change),
                    }),
                    Instruction::M5(M5 {}),
                    Instruction::M6(M6 {
                        t: self.tool_ordering(tool).unwrap(),
                    }),
                    Instruction::S(S {x: tool.spindle_speed()}),
                    if tool.direction() == Direction::Clockwise {
                        Instruction::M3(M3 {})
                    } else {
                        Instruction::M4(M4 {})
                    }
                ]);

                // Add tool instructions
                raw_instructions.append(&mut locked_context.to_instructions());
            }
        }

        // End program
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

        instructions
    }

    pub fn to_gcode(&self) -> String {
        self.to_instructions().iter()
            .map(|instruction| instruction.to_gcode())
            .collect::<Vec<String>>()
            .join("\n")
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
    fn test_program_tools() {
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
                1.0
            ));
        });

        program.extend(tool2, |context| {
            context.append_cut(Cut::path(
                Vector3::new(5.0, 10.0, 3.0),
                vec![Segment::line(Vector2::new(5.0, 10.0), Vector2::new(15.0, 10.0))],
                -0.1,
                1.0
            ));
        });

        let tools = program.tools();
        assert_eq!(tools, vec![tool1, tool2]);

        program.set_tool_ordering(tool2, 0);

        let tools = program.tools();
        assert_eq!(tools, vec![tool2, tool1]);
    }

    #[test]
    fn test_program_to_instructions() {
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
                1.0
            ));
        });

        program.extend(tool2, |context| {
            context.append_cut(Cut::path(
                Vector3::new(5.0, 10.0, 3.0),
                vec![Segment::line(Vector2::new(5.0, 10.0), Vector2::new(15.0, 10.0))],
                -0.1,
                1.0
            ));
        });

        let instructions = program.to_instructions();

        let expected_output = vec![
            Instruction::G21(G21 {}),
            Instruction::Empty(Empty {}),
            Instruction::Message(Message { text: "Tool change: Cylindrical tool: diameter = 4mm, length = 50mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min".to_string() }),
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
            Instruction::Message(Message { text: "Tool change: Conical: angle = 45°, diameter = 15mm, length = 18.1066mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min".to_string() }),
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
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ];

        assert_eq!(instructions, expected_output);

        program.set_tool_ordering(tool2, 1);

        let instructions = program.to_instructions();

        let expected_output = vec![
            Instruction::G21(G21 {}),
            Instruction::Empty(Empty {}),
            Instruction::Message(Message { text: "Tool change: Conical: angle = 45°, diameter = 15mm, length = 18.1066mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min".to_string() }),
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
            Instruction::Message(Message { text: "Tool change: Cylindrical tool: diameter = 4mm, length = 50mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min".to_string() }),
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
            Instruction::Empty(Empty {}),
            Instruction::M2(M2 {}),
        ];

        assert_eq!(instructions, expected_output);
    }

    #[test]
    fn test_program_to_gcode() {
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
                1.0
            ));
        });

        program.extend(tool2, |context| {
            context.append_cut(Cut::path(
                Vector3::new(5.0, 10.0, 3.0),
                vec![Segment::line(Vector2::new(5.0, 10.0), Vector2::new(15.0, 10.0))],
                -0.1,
                1.0
            ));
        });

        program.set_tool_ordering(tool2, 1);

        let gcode = program.to_gcode();

        let expected_output = vec![
            "G21".to_string(),
            "".to_string(),
            "(MSG,Tool change: Conical: angle = 45°, diameter = 15mm, length = 18.1066mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min)".to_string(),
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
            "(MSG,Tool change: Cylindrical tool: diameter = 4mm, length = 50mm, direction = clockwise, spindle_speed = 5000, feed_rate = 400mm/min)".to_string(),
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
            "".to_string(),
            "M2".to_string(),
        ].join("\n");

        assert_eq!(gcode, expected_output);
    }
}
