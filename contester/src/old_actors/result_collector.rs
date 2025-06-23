use actix::prelude::*;
use std::collections::HashMap;
use crate::actors::messages::{TaskResult};

pub struct ResultCollector {
    results: HashMap<u64, TaskResult>,
}

impl Actor for ResultCollector {
    type Context = Context<Self>;
}

impl Default for ResultCollector {
    fn default() -> Self {
        Self {
            results: HashMap::new(),
        }
    }
}

// Message handler for TaskResult
impl Handler<TaskResult> for ResultCollector {
    type Result = String;

    fn handle(&mut self, msg: TaskResult, _ctx: &mut Self::Context) -> Self::Result {
        // TODO: Implement actual result storage and processing logic
        // This should:
        // 1. Store the result in a persistent storage
        // 2. Update any metrics or statistics
        // 3. Notify any subscribers about the new result
        // 4. Handle any errors that might occur

        // For now, just store the result in memory
        self.results.insert(msg.task_id, msg.clone());
        println!("Stored result for task {}", msg.task_id);
        "OK".to_string()
    }
}

// Add a method to get results
impl ResultCollector {
    pub fn get_result(&self, task_id: u64) -> Option<&TaskResult> {
        self.results.get(&task_id)
    }
}
