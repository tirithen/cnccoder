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
    tool_ordering: Arc<Mutex<HashMap<Tool, i32>>>,
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

    pub fn tool_ordering(&self, tool: Tool) -> Option<i32> {
        if let Some(ordering) = self.tool_ordering.lock().unwrap().get(&tool) {
            return Some(*ordering);
        }

        None
    }

    pub fn set_tool_ordering(&self, tool: Tool, ordering: i32) {
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
        orderings.sort_by(|a, b| b.1.cmp(a.1));

        for (tool, _) in orderings {
            tools.push(*tool);
        }

        tools
    }

    pub fn to_instructions(&self) -> Vec<Instruction> {
        let contexts = self.contexts.lock().unwrap();
        let tools = self.tools();

        let mut raw_instructions = vec![];
        for tool in tools {
            if let Some(context) = contexts.get(&tool) {
                let locked_context = &mut context.lock().unwrap();
                raw_instructions.append(&mut locked_context.to_instructions());
            }
        }

        let raw_length = raw_instructions.len();
        let mut instructions = vec![];
        for (index, instruction) in (&raw_instructions).iter().enumerate() {
            if (index == 0 || index == raw_length - 1) && *instruction == Instruction::Empty(Empty {}) {
                continue;
            }

            if index < raw_length - 1 && instruction == &raw_instructions[index + 1] {
                continue;
            }

            instructions.push(instruction.clone());
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_program() {
        let program = Program::new(Units::Metric, 10.0, 50.0);
        assert!(program.z_safe == 10.0);
        assert!(program.z_tool_change == 50.0);
    }

    fn vec_compare<T>(va: &[T], vb: &[T]) -> bool
        where T: PartialEq {
        (va.len() == vb.len()) &&
         va.iter()
           .zip(vb)
           .all(|(a,b)| *a == *b)
    }

    #[test]
    fn test_program_to_instructions() {
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
                Vector3::default(),
                vec![Segment::line(Vector2::default(), Vector2::new(5.0, 10.0))],
                -0.1,
                1.0
            ));
        });

        let instructions = program.to_instructions();

        let expected_output = vec![
            Instruction::Comment(Comment { text: "Cut path at: x = 0, y = 0".to_string() }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) }),
            Instruction::G0(G0 { x: Some(0.0), y: Some(0.0), z: None }),
            Instruction::G1(G1 { x: None, y: None, z: Some(0.0), f: Some(400.0) }),
            Instruction::G1(G1 { x: Some(0.0), y: Some(0.0), z: Some(-0.1), f: None }),
            Instruction::G1(G1 { x: Some(5.0), y: Some(10.0), z: Some(-0.1), f: None }),
            Instruction::G0(G0 { x: None, y: None, z: Some(10.0) })
        ];

        assert!(vec_compare(&instructions, &expected_output));
    }
}
