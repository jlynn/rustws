#![feature(thread_id_value)]

use actix_web::{web, App, Error, HttpResponse, HttpRequest, HttpServer, Responder};
use bytes::Bytes;
use futures::stream::once;
use futures::future::ok;
use listenfd::ListenFd;
use std::thread;

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {} from {}!", &name, thread::current().id().as_u64())

}

async fn chat() -> HttpResponse {
    let body = once(ok::<_, Error>(Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/chat", web::to(chat))
            .route("/{name}", web::get().to(greet))
    })
    .keep_alive(60);

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8080")?
    };

    server.run().await
}
