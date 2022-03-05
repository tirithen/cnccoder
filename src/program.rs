use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;

use serde::{Deserialize, Serialize};

use crate::cuts::*;
use crate::tools::*;
use crate::types::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Operation {
    Cut(Cut),
    // Tool(Tool),
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
}

pub struct Program {
    pub z_safe: f64,
    pub z_tool_change: f64,
    units: Units,
    contexts: HashMap<Tool, Context>,
    tools_priorities: HashMap<Tool, i32>,
}

impl Program {
    pub fn new(units: Units, z_safe: f64, z_tool_change: f64) -> Program {
        Program {
            z_safe,
            z_tool_change,
            units,
            contexts: HashMap::new(),
            tools_priorities: HashMap::new(),
        }
    }

    fn append(&mut self, tool: Tool, operation: Operation) -> Result<(), String> {
        if let Vacant(entry) = self.contexts.entry(tool) {
            let context = Context::new(self.units, tool, self.z_safe, self.z_tool_change);
            self.tools_priorities.insert(tool, 0);
            entry.insert(context);
        }

        let context = self.contexts.get_mut(&tool).ok_or("Failed to fetch context")?;
        context.operations.push(operation);

        Ok(())
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
    fn test_program_append() {
        let mut program = Program::new(Units::Metric, 10.0, 50.0);

        let tool = Tool::Cylindrical(Cylindrical::new(Units::Metric, 50.0, 4.0));
        let operation = Operation::Cut(Cut::Line(Line::new(
            Vector2::default(),
            Vector2::new(5.0, 10.0)
        )));

        let result = program.append(tool, operation);

        assert!(result.is_ok());
    }
}
