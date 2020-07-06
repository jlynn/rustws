use std::collections::HashMap;

use actix::{Actor, Context};
use actix::prelude::*;
use actix_web::{web, Error};
use actix_web::error::PayloadError;
use actix_web_actors::HttpContext;
use bytes::Bytes;

#[derive(Message)]
#[rtype(result = "()")]
struct Message {
    pub from: String,
    pub msg: String,
}

#[derive(Message)]
#[rtype(result = "String")]
struct Join {
    name: String,
    addr: Recipient<Message>,
}

pub struct ChatServer {
    sessions: HashMap<String, Recipient<Message>>,
}

impl Default for ChatServer {
    fn default() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
        }
    }
}

impl ChatServer {
    fn send_message(&self, msg: Message) {
        for k in self.sessions.keys() {
            if &msg.from != k {
                println!("Sending {} to {}", msg.msg, k);
            }
        }
    }
}

impl Actor for ChatServer {
    // Give ChatServer the ability to communicate with other actors
    type Context = Context<Self>;
}

impl Handler<Join> for ChatServer {
    type Result = String;

    fn handle(&mut self, join: Join, _: &mut Context<Self>) -> Self::Result {
        println!("{} joined the conversation", join.name);
        join.name
    }
}

impl Handler<Message> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) -> Self::Result {
        println!("Received a message from {}\n", msg.from);
    }
}

pub struct ChatClient {
    pub name: String,
    pub srv: Addr<ChatServer>,
}

impl ChatClient {
    pub fn create(
        name: String,
        srv: Addr<ChatServer>,
        stream: web::Payload
    ) -> impl Stream<Item = Result<Bytes, Error>> {
        let response_stream = HttpContext::with_factory(|ctx| {
            ctx.add_stream(stream);
            ChatClient {
                name: name,
                srv: srv,
            }
        });
        response_stream
    }

    fn send_message(&mut self, msg: Message, ctx: &mut HttpContext<Self>) {
        let data = format!("{}: {}", msg.from, msg.msg);
        ctx.write(Bytes::from(data));
    }
}

impl Actor for ChatClient {
    // Give ChatClient context for handling http streams
    type Context = HttpContext<Self>;

    fn started(&mut self, ctx: &mut HttpContext<Self>) {
        // Register with chat server
        let addr = ctx.address();
        self.srv.send(
            Join {
                name: self.name.clone(),
                addr: addr.recipient(),
            }
        )
        .into_actor(self)
        .then(|res, act, ctx| {
            match res {
                Ok(res) => act.name = res,
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx)
    }
}

impl Handler<Message> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        self.send_message(msg, ctx);
    }
}

impl StreamHandler<Result<Bytes, PayloadError>> for ChatClient {
    fn handle(&mut self, msg: Result<Bytes, PayloadError>, _ctx: &mut Self::Context) {
        println!("Received a message from {}: {}", self.name, msg.unwrap().len())
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("finished");
    }
}
