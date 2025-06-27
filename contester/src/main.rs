mod actors;
use actix::prelude::*;
use actix_web::{App, HttpServer, http::header::q, web};
use futures_util::StreamExt;
//use futures_util::io::AsyncWriteExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::client::conn::http1::{Connection, SendRequest};
use hyper::{Method, Request, upgrade};
use hyper_util::rt::TokioIo;
use podman_api::{Podman, api::Exec, opts::*};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct CreateContainerResponse {
    id: String,
    warnings: Vec<String>,
}

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

async fn get_podman_conn() -> Result<
    (
        SendRequest<Full<Bytes>>,
        Connection<TokioIo<UnixStream>, Full<Bytes>>,
    ),
    Box<dyn std::error::Error>,
> {
    let path = Path::new("/run/user/1000/podman/podman.sock");
    let stream = UnixStream::connect(path).await?;
    let io = TokioIo::new(stream);
    let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    Ok((sender, conn))
}

async fn podman_test() {
    //let podman = Podman::new("unix:///run/user/1000/podman/podman.sock");

    let (mut sender, conn) = get_podman_conn().await.unwrap(); // TODO:
    tokio::spawn(async move {
        // Этому соединению не нужен upgrade
        if let Err(err) = conn.with_upgrades().await {
            eprintln!("Соединение 1 (create/start) разорвано: {:?}", err);
        }
    });

    // 4. Создаем наш JSON-пейлоад
    let json_payload = json!({
      "Image": "myapp:latest",
      "resource_limits": {
        "memory": {
          "limit": 134217728,
          "swap": 268435456
        },
        "cpu": {
          "quota": 50000,
          "period": 100000
        }
      },
      "timeout": 10,
      "tty": false,
      "stdin": true,
      //"Cmd": ["cat"]
      //"Cmd": ["/bin/sh", "/app.sh"]
    });
    let body_bytes = json_payload.to_string();
    println!("Подсосались");

    let podman = Podman::new("unix:///run/user/1000/podman/podman.sock").unwrap(); // TODO::
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

    // 5. Формируем HTTP POST-запрос вручную
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://localhost/v4.0.0/libpod/containers/create")
        .header("Content-Type", "application/json")
        .header("Host", "localhost") // Этот заголовок требуется для hyper
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap(); // TODO:

    println!("Отправка запроса на создание контейнера...");

    // 6. Отправляем запрос и ждем ответ
    let response = sender.send_request(req).await.unwrap(); // TODO:

    println!("Получен ответ: {}", response.status());
    assert_eq!(response.status(), hyper::StatusCode::CREATED); // Ожидаем статус 201

    // 7. Читаем и разбираем тело ответа
    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap() // TODO:
        .to_bytes();
    let create_response: CreateContainerResponse = serde_json::from_slice(&body_bytes).unwrap(); // TODO:

    println!("Контейнер создан! ID: {}", create_response.id.clone());

    println!("Шаг 3: Подключение к потокам (attach)...");
    let attach_uri = format!(
        "http://localhost/v4.0.0/libpod/containers/{}/attach?stdin=true&stdout=true&stream=true",
        create_response.id.clone()
    );

    let mut attach_req = Request::builder()
        .method(Method::POST)
        .uri(attach_uri)
        .header("Host", "localhost")
        .header("Connection", "Upgrade")
        .header("Upgrade", "tcp") // Стандартные заголовки для запроса на 'upgrade'
        .body(Full::new(Bytes::new()))
        .unwrap(); // TODO:

    let res = sender.send_request(attach_req).await.unwrap(); // TODO:

    // Проверяем, что сервер согласился на 'upgrade'
    if res.status() != hyper::StatusCode::SWITCHING_PROTOCOLS {
        panic!("Server did not switch protocols. Status: {}", res.status());
    }

    println!("Шаг 4: Разделение потока на reader и writer...");
    let upgraded_connection = upgrade::on(res).await;
    if let Err(e) = upgraded_connection {
        println!("{:#?}", e);
        return;
    }
    let upgraded_connection = upgraded_connection.unwrap();
    let io = TokioIo::new(upgraded_connection);
    let (reader, mut writer) = io::split(io);

    println!("Поток успешно разделён!");

    println!("Шаг 2: Запуск контейнера...");
    let (mut sender, conn) = get_podman_conn().await.unwrap(); // TODO:
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Соединение 2 (start) разорвано: {:?}", e);
        }
    });
    let start_req = Request::builder()
        .method(Method::POST)
        .uri(format!(
            "http://localhost/v4.0.0/libpod/containers/{}/start",
            create_response.id.clone()
        ))
        .header("Host", "localhost")
        .body(Full::new(Bytes::new()))
        .unwrap(); // TODO:

    sender.send_request(start_req).await.unwrap(); // TODO:
    println!("Контейнер запущен");

    println!("Шаг 5. Ожидание готовности скрипта");
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        if buf_reader.read_line(&mut line).await.unwrap() == 0 {
            // TODO:
            panic!("Скрипт завершился, не отправив сигнал READY"); // TODO:
        }
        if line.contains("READY") {
            println!("Скрипт готов!");
            line.clear(); // для следующего чтения
            break;
        } else {
            println!("Неверный ввод скрипта: {}", line);
        }
        line.clear(); // для следующей итерации
    }

    println!("Шаг 6: Отправка данных и получение ответа...");
    let test_message = "Финишная прямая!\n";
    writer.write_all(test_message.as_bytes()).await.unwrap(); // TODO:
    writer.flush().await.unwrap(); // TODO:
    println!("   Отправлено: {}", test_message.trim());

    // Читаем эхо-ответ от скрипта
    buf_reader.read_line(&mut line).await.unwrap(); // TODO:
    println!("   Получено:   {}", line.trim());

    drop(writer);

    println!("Шаг 7: Остановка и удаление контейнера...");
    let (mut sender, conn) = get_podman_conn().await.unwrap(); // TODO:
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Соединение 3 (kill/remove) разорвано: {:?}", err);
        }
    });
    let kill_req = Request::builder()
        .method(Method::POST)
        .uri(format!(
            "http://localhost/v4.0.0/libpod/containers/{}/kill",
            create_response.id.clone()
        ))
        .header("Host", "localhost")
        .body(Full::new(Bytes::new()))
        .unwrap(); // TODO:
    sender.send_request(kill_req).await.unwrap(); // TODO:

    println!("Контайнер остановлен");

    let remove_req = Request::builder()
        .method(Method::DELETE)
        .uri(format!(
            "http://localhost/v4.0.0/libpod/containers/{}",
            create_response.id.clone()
        ))
        .header("Host", "localhost")
        .body(Full::new(Bytes::new()))
        .unwrap();
    sender.send_request(remove_req).await.unwrap();
    println!("Контейнер удалён. Готово.");
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
