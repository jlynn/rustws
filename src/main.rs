#![feature(thread_id_value)]

use actix::{Actor, Addr};
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use listenfd::ListenFd;

mod chat;

async fn chat(
    name: web::Path<String>,
    stream: web::Payload,
    srv: web::Data<Addr<chat::ChatServer>>,
) -> Result<HttpResponse, Error> {

    let client_name = name.to_string();
    Ok(HttpResponse::Ok().streaming(
        chat::ChatClient::create(
            client_name,
            srv.get_ref().clone(),
            stream,
        )
    ))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();

    let chat_server = chat::ChatServer::default().start();

    let mut server = HttpServer::new(move || {
        App::new()
            .data(chat_server.clone())
            .route("/chat/{name}", web::post().to(chat))
    })
    .keep_alive(60);

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8080")?
    };

    server.run().await
}
