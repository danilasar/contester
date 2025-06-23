use actix::prelude::*;
use crate::actors::{worker::Worker, messages::{EvaluateTask, TaskResult}};

pub struct TaskManager;

impl Actor for TaskManager {
    type Context = Context<Self>;
}

// Message handler for EvaluateTask
impl Handler<EvaluateTask> for TaskManager {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: EvaluateTask, ctx: &mut Self::Context) -> Self::Result {
        // Create a new Worker instance
        let worker = Worker.start();
        
        // Send the task to the worker
        Box::pin(async move {
            // Send the task to the worker and await the result
            worker.send(msg).await.unwrap()
        })
    }
}
