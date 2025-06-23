use actix::prelude::*;
use containerd_client::Client;

struct ValidatorSpec {
    mode: ValidatorMode,
    tolerance: Option<f64>,
}

enum ValidatorMode {
    Exact,
    Approximate,
    CustomCode(String)
}

struct OutputValidator

impl Actor for OutputValidator {
    type Context = Context<Self>;
}

impl Handler<ValidateOutput> for OutputValidator {
    type Result = ValidationResult;

    fn handle(&mut self, msg: ValidateOutput, _ctx: &mut Self::Context) -> Self::Result {
        match msg.spec.mode {
            ValidatorMode::Exact => ValidationResult {
                passed: msg.output == msg.expected,
                score: if msg.output == msg.expected { 1.0 } else { 0.0 }
            },
            ValidatorMode::Approximate => {
                // TODO: make more universal
                let output: f64 = msg.output.parse().unwrap();
                let expected: f64 = msg.expected.parse().unwrap();
                let tolerance = msg.spec.tolerance.unwrap();
                ValidationResult {
                    passed: (output - expected).abs() <= tolerance,
                    score: if (output - expected).abs() <= tolerance { 1.0 } else { 0.0 }
                }
            },
            ValidatorMode::Custom => {
                let client = Client::connet("/run/containerd/containerd.sock").unwrap();
                let container = client.create_container("validator", "dockr.io/library/python:latest").unwrap();
                // container.copy_to("/validator.py", msg.spec.custom_code.unwrap().as_bytes()).unwrap();
                // let result = container.exec("python /validator.py").unwrap();
                // TODO:
                ValidationResult {
                    passed: true,
                    score: 1.0
                }
            }
        }
    }
}
