use std::net::SocketAddr;

use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    routing::get,
    Router,
    response,
};
use mini_redis::{DEFAULT_ADDR, FilterLayer, BASE_REQUEST};
use serde::Deserialize;
use std::sync::Arc;
use volo_gen::mini_redis::{RedisServiceClient, RedisRequest,RequestType, RedisResponse};
use faststr::FastStr;
type RpcClient = RedisServiceClient;

async fn get_response(req: RedisRequest, rpc_cli: &mut RpcClient) -> Option<FastStr> {
    rpc_cli.redis_command(req).await.unwrap_or_else(|x|  RedisResponse{ 
        value: Some(FastStr::from(match x{
            volo_thrift::ResponseError::Application(err) => err.message,
            _ => "Internal Server Error".to_string(),
            })),
        response_type: volo_gen::mini_redis::ResponseType::Print,
    }).value
}


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let rpc_cli =volo_gen::mini_redis::RedisServiceClientBuilder::new("redis")
        // .layer_outer(LogLayer)
        .layer_outer(FilterLayer)
        .address(addr)
        .build();

    // build the application with router
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/get/:keys", get(get_key).with_state(rpc_cli.clone()))
        .route(
            "/set",
            get(set_key).with_state(rpc_cli.clone()),
        )
        .route("/del/:keys", get(del_key).with_state(rpc_cli));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}

/// Get a key
async fn get_key(Path(key): Path<String>, State(mut rpc_cli): State<RpcClient>) -> Result<response::Json<String>, StatusCode> {
    let req = RedisRequest {
        key: Some(FastStr::from(Arc::new(key))),
        request_type: RequestType::Get,
        ..BASE_REQUEST.clone()
    };
    match get_response(req, &mut rpc_cli).await {
        Some(value) => match value.as_str(){
            "nil" => Ok(response::Json("nil".to_string())),
            _ => Ok(response::Json(value.into_string())),
        },
        None => Ok(response::Json("nil".to_string())),
    }
}

#[derive(Deserialize)]
struct KVPair {
    key: String,
    value: String,
}

async fn set_key(State(mut rpc_cli): State<RpcClient>, Query(params): Query<KVPair>) -> Result<response::Json<String>,StatusCode> {
    let req = RedisRequest {
        key: Some(FastStr::from(Arc::new(params.key))),
        value: Some(FastStr::from(Arc::new(params.value))),
        request_type: RequestType::Set,
        ..BASE_REQUEST.clone()
    };
    match get_response(req, &mut rpc_cli).await {
        Some(value) => Ok(response::Json(value.into_string())),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn del_key(
    State(mut rpc_cli): State<RpcClient>,
    Path(key): Path<String>,
) -> Result<response::Json<String>, StatusCode> {
    let req = RedisRequest {
        key: Some(FastStr::from(Arc::new(key))),
        request_type: RequestType::Del,
        ..BASE_REQUEST.clone()
    };
    match get_response(req, &mut rpc_cli).await {
        Some(value) => {
            match value.as_str() {
                "Ok" => Ok(response::Json(value.into_string())),
                "nil" => Ok(response::Json("nil".to_string())),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
