use actix::Message;

#[derive(Clone, Message)]
#[rtype(result = "TaskResult")]
pub struct EvaluateTask {
    pub task_id: u64,
    pub solution_code: String,
    pub generator: Option<GeneratorSpec>,
    pub validator: ValidatorSpec
}

#[derive(Clone, Message)]
#[rtype(result = "String")]
pub struct TaskResult {
    pub task_id: u64,
    pub result: String,
}
