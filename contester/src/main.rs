mod actors;
use actix::prelude::*;
use actix_web::{App, HttpServer, http::header::q, web};
use futures_util::StreamExt;
use futures_util::io::AsyncWriteExt;
use podman_api::{Podman, api::Exec, opts::*};

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

async fn podman_test() {
    let podman = Podman::new("unix:///run/user/1000/podman/podman.sock");
    if let Err(_) = podman {
        // TODO:
        return;
    }
    let podman = podman.unwrap(); // if let Err upper
    let build_opts =
        ImageBuildOpts::builder("/data/docs/sgu/coursework/contester/containers/helloworld")
            .dockerfile("Dockerfile")
            .tag("myapp:latest")
            .build();
    match podman.images().build(&build_opts) {
        Ok(mut build_stream) => {
            while let Some(chunk) = build_stream.next().await {
                match chunk {
                    Ok(chunk) => println!("{:?}", chunk),
                    Err(e) => {
                        eprintln!("Chunk build error: {}", e);
                        return;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Build error: {}", e);
        }
    };

    println!("abaunda");

    let linux_resources = podman_api::models::LinuxResources {
        memory: Some(podman_api::models::LinuxMemory {
            limit: Some(134217728), // 128 MB
            swap: Some(268435456),  // 256 MB total
            swappiness: None,
            reservation: None,        // No soft limit
            disable_oom_killer: None, // Default OOM behavior
            use_hierarchy: None,      // Default hierarchy
            kernel: None,             // No kernel memory limit
            kernel_tcp: None,         // No TCP kernel limit
        }),
        cpu: Some(podman_api::models::LinuxCpu {
            quota: Some(50000), // 50% CPU
            period: Some(100000),
            shares: None,           // No CPU shares
            realtime_runtime: None, // No realtime
            realtime_period: None,  // No realtime period
            cpus: None,             // No CPU pinning
            mems: None,             // No memory nodes
        }),
        devices: None,         // No device limits
        pids: None,            // No PID limits
        block_io: None,        // No block I/O limits
        hugepage_limits: None, // No hugepage limits
        network: None,         // No network limits
        rdma: None,            // No RDMA limits
        unified: None,         // No unified cgroup limits
    };

    let container_opts = ContainerCreateOpts::builder()
        .image("myapp:latest")
        //.command(vec!["/bin/bash"])
        .resource_limits(linux_resources)
        .build();

    let container = podman.containers().create(&container_opts).await;
    if let Err(e) = container {
        // TODO:
        return;
    }
    println!("Контайнер создан");
    let container = container.unwrap(); // if let Err upper
    for warning in container.warnings {
        eprintln!("{}", warning);
    }
    let container = podman.containers().get(container.id);
    println!("Нашли наш контейнер");
    /*let exec_start_opts = ExecStartOpts::builder()
    .tty(true)
    .build();*/
    let exec_opts = ExecCreateOpts::builder()
        .attach_stdin(true)
        .attach_stdout(true)
        .attach_stderr(true)
        .tty(true)
        .build();

    let exec = container.exec(&exec_opts).await;

    let exec_start_opts = ExecStartOpts::builder().tty(true).build();

    let exec_stream = exec.start(&exec_start_opts).await?;
    if let Err(e) = container.start(None).await {
        eprintln!("{:#?}", e);
        // TODO:
        return;
    }
    println!("Запустили контейнер");

    let attach_opts = ContainerAttachOpts::builder()
        .stdin(true)
        .stdout(true)
        .stderr(true)
        .build();

    let tty_multiplexer = container.attach(&attach_opts).await;
    if let Err(e) = tty_multiplexer {
        eprintln!("{:#?}", e);
        // TODO:
        return;
    }
    println!("Подсосались к контейнеру");
    let (mut reader, mut writer) = tty_multiplexer.unwrap().split(); // if let Err upper

    let logs_opts = ContainerLogsOpts::builder()
        .stdout(true)
        .stderr(true)
        .build();
    let mut logs = container.logs(&logs_opts);
    // Читаем начальные логи
    println!("Reading initial logs:");
    while let Some(log_chunk) = logs.next().await {
        match log_chunk {
            Ok(chunk) => {
                println!("Log chunk: {:?}", String::from_utf8_lossy(&chunk));
            }
            Err(e) => {
                eprintln!("Log error: {}", e);
                break;
            }
        }
    }

    // Читаем начальный вывод
    let mut output_buffer = Vec::new();
    for _ in 0..3 {
        // Читаем несколько chunk'ов
        if let Some(Ok(chunk)) = reader.next().await {
            output_buffer.extend_from_slice(&chunk);
            println!("Received: {:?}", String::from_utf8_lossy(&chunk));
        }
    }

    // Отправляем ответ на read
    writer.write_all(b"Hello from Rust\n").await;
    writer.flush().await;

    // Продолжаем читать
    for _ in 0..5 {
        // Читаем еще несколько chunk'ов
        if let Some(Ok(chunk)) = reader.next().await {
            println!("After input: {:?}", String::from_utf8_lossy(&chunk));
        }
    }
    if let Err(e) = writer.write_all(b"echo 'Hello from Rust'\n").await {
        eprintln!("{:#?}", e);
    }
    println!("Вдули контейнеру");
    while let Some(tty_result) = reader.next().await {
        match tty_result {
            Ok(chunk) => println!("{:#?}", chunk),
            Err(e) => eprintln!("Error: {:#?}", e),
        }
    }
    println!("Отсосали у контейнера");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    podman_test().await;
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
