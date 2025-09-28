use plugin_manager::Plugin;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskA;

impl Plugin for TaskA {
    fn name(&self) -> String {
        String::from("task_a")
    }

    fn execute(&self, _context: &dyn Any) -> Result<(), Box<dyn std::error::Error>> {
        println!("Executing Task A");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl TaskA {
    pub fn other_method(&self) {
        println!("Executing other method in Task A");
    }
}
