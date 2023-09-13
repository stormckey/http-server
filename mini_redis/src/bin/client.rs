use lazy_static::lazy_static;
use mini_redis::FilterLayer;
use std::net::SocketAddr;
use std::sync::Arc;
use volo::FastStr;
use volo_gen::mini_redis::{RedisRequest, RequestType};
use mini_redis::{DEFAULT_ADDR,BASE_REQUEST};

lazy_static! {
    static ref CLIENT: volo_gen::mini_redis::RedisServiceClient = {
        let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
        volo_gen::mini_redis::RedisServiceClientBuilder::new("redis")
            // .layer_outer(LogLayer)
            .layer_outer(FilterLayer)
            .address(addr)
            .build()
    };
}
#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut args: Vec<String> = std::env::args().collect();

    let req = match args[1].to_lowercase().as_str() {
        "set" => RedisRequest {
            key: Some(FastStr::from(Arc::new(args.remove(2)))),
            value: Some(FastStr::from(Arc::new(args.remove(2)))),
            request_type: RequestType::Set,
            expire_time: match args.len() {
                4 => Some(args.remove(3).parse().unwrap()),
                2 => None,
                _ => panic!("invalid args"),
            },
            ..BASE_REQUEST.clone()
        },
        "get" => RedisRequest {
            key: Some(FastStr::from(Arc::new(args.remove(2)))),
            request_type: RequestType::Get,
            ..BASE_REQUEST.clone()
        },
        "del" => RedisRequest {
            key: Some(FastStr::from(Arc::new(args.remove(2)))),
            request_type: RequestType::Del,
            ..BASE_REQUEST.clone()
        },
        "ping" => RedisRequest { ..BASE_REQUEST.clone() },
        "subscribe" => RedisRequest {
            request_type: RequestType::Subscribe,
            channels: Some(
                args.drain(2..)
                    .map(|x| FastStr::from(Arc::new(x)))
                    .collect(),
            ),
            block: Some(false),
            ..BASE_REQUEST.clone()
        },
        "publish" => RedisRequest {
            value: Some(FastStr::from(Arc::new(args.remove(3)))),
            request_type: RequestType::Publish,
            channels: Some(vec![FastStr::from(Arc::new(args.remove(2)))]),
            ..BASE_REQUEST.clone()
        },
        _ => {
            panic!("unknown command");
        }
    };
    let resp = CLIENT.redis_command(req.clone()).await;
    match resp {
        Ok(resp) => match resp.response_type {
            volo_gen::mini_redis::ResponseType::Print => {
                println!("{}", resp.value.unwrap())
            }
            volo_gen::mini_redis::ResponseType::Trap => {
                println!(
                    "subscribe {} channels",
                    req.channels.as_ref().unwrap().len()
                );
                loop {
                    let req = RedisRequest {
                        block: Some(true),
                        ..req.clone()
                    };
                    let resp = CLIENT.redis_command(req).await;
                    match resp {
                        Ok(info) => {
                            println!("{}", info.value.unwrap());
                        }
                        _ => {
                            println!("error");
                        }
                    }
                }
            }
        },
        Err(e) => match e {
            volo_thrift::ResponseError::Application(err) => {
                println!("{}", err.message)
            }
            _ => {
                tracing::error!("{:?}", e);
            }
        },
    }
}
