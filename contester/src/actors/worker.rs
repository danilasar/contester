use actix::prelude::*;
use crate::actors::{result_collector::ResultCollector, messages::{EvaluateTask, TaskResult}};

pub struct Worker;

impl Actor for Worker {
    type Context = Context<Self>;
}

// Message handler for EvaluateTask
impl Handler<EvaluateTask> for Worker {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: EvaluateTask, ctx: &mut Self::Context) -> Self::Result {
        // TODO: Implement actual task evaluation logic here
        // This should:
        // 1. Run the solution code in a safe container
        // 2. Capture the output and execution time
        // 3. Handle any errors that might occur
        // 4. Generate a proper TaskResult with actual results

        // For now, create a mock TaskResult
        let result = TaskResult {
            task_id: msg.task_id,
            result: format!("Mock result for task {}", msg.task_id),
        };

        // Send the result to ResultCollector
        let addr = ResultCollector::from_registry();
        addr.do_send(result.clone());

        // Return the result
        Box::pin(async { result })
    }
}
