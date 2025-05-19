mod actors;

use actix::prelude::*;
use actix_web::{web, App, HttpServer};
use actors::{
    task_manager::TaskManager,
    worker::Worker,
    container_manager::ContainerManager,
    result_collector::ResultCollector,
    messages::{EvaluateTask, TaskResult},
};

// Define our actor
struct MyActor;

impl Actor for MyActor {
    type Context = Context<Self>;
}

// Define a message
#[derive(Message)]
#[rtype(result = "String")]
struct Ping;

// Implement message handler
impl Handler<Ping> for MyActor {
    type Result = String;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        "Pong".to_string()
    }
}

// Create a web endpoint that uses our actor
async fn ping_handler(addr: web::Data<Addr<MyActor>>) -> String {
    addr.send(Ping).await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start actor system
    let addr = MyActor.start();

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(addr.clone()))
            .route("/ping", web::get().to(ping_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
