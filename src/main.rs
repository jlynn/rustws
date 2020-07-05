#![feature(thread_id_value)]

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use actix::{Actor, AsyncContext};
use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder};
use actix_web_actors::HttpContext;
use bytes::Bytes;
//use futures::stream::once;
//use futures::future::ok;
use listenfd::ListenFd;

struct ChatActor {
    counter: usize,
}

struct Count {
    counter: Mutex<i32>,
}

impl Actor for ChatActor {
    type Context = HttpContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_later(Duration::from_millis(2000), |slf, ctx| slf.write(ctx));
    }
}

impl ChatActor {
    fn write(&mut self, ctx: &mut HttpContext<Self>) {
        self.counter += 1;
        if self.counter > 3 {
            ctx.write_eof()
        } else {
            ctx.write(Bytes::from(format!("LINE-{}\r\n", self.counter)));
            ctx.run_later(Duration::from_millis(2000), |slf, ctx| slf.write(ctx));
        }
    }
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {} from {}!", &name, thread::current().id().as_u64())

}

async fn count(data: web::Data<Count>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("The count from {} is {}", thread::current().id().as_u64(), counter)
}

async fn chat() -> HttpResponse {
    HttpResponse::Ok()
        .streaming(HttpContext::create(ChatActor { counter: 0 }))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let counter = web::Data::new(Count {
        counter: Mutex::new(0),
    });
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .route("/", web::get().to(greet))
            .route("/chat", web::to(chat))
            .route("/count", web::to(count))
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
