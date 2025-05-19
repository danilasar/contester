use actix::Message;

#[derive(Message)]
#[rtype(result = "TaskResult")]
pub struct EvaluateTask {
    pub task_id: u64,
    pub solution_code: String,
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct TaskResult {
    pub task_id: u64,
    pub result: String,
}
