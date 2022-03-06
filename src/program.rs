use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cuts::*;
use crate::tools::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub enum Operation {
    Cut(Cut),
}

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
}

pub struct Program {
    pub z_safe: f64,
    pub z_tool_change: f64,
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

    #[test]
    fn test_program_extend() {
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
    }
}
