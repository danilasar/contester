use actix::prelude::*;

struct GeneratorSpec {
    code: Option<String>,
    params: HashMap<String, String>
}

struct DataGenerator;

impl Actor for DataGenerator {
    type Context = Context<Self>;
}


impl Handler<GenerateInput> for DataGenerator {
    type Result = String;

    fn handle(&mut self, msg: GenerateInput, _ctx: &mut Self::Context) -> Self::Result {
        let n: usize = msg.spec.params.get("n").unwrap().parse().unwrap();
        let max_val: i32 = msg.spec.params.get("max_val").unwrap().parse().unwrap();
        let mut rng = rand::thread_rng();
        (0..n).map(|_| rng.gen_range(0..max_val).to_string()).collect::<Vec<_>>().join("\n")
    }
}
