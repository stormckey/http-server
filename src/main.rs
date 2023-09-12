use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Form, Router,
};
use rpcdemo::DEFAULT_ADDR;
use serde::Deserialize;
use volo_gen::demo::{DemoServiceClient, DemoServiceClientBuilder};

type RpcClient = DemoServiceClient;
type RpcClientBuilder = DemoServiceClientBuilder;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let rpc_cli = RpcClientBuilder::new("rpcdemo").address(addr).build();

    // build the application with router
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/get/:keys", get(get_key).with_state(rpc_cli.clone()))
        .route(
            "/set",
            get(show_set_form).post(set_key).with_state(rpc_cli.clone()),
        )
        .route("/del", get(show_del_form).post(del_key).with_state(rpc_cli));

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
async fn get_key(Path(key): Path<String>, State(rpc_cli): State<RpcClient>) -> Response {
    if rpc_cli.get(key.into()).await.unwrap() {
        (StatusCode::OK, "found").into_response()
    } else {
        (StatusCode::NOT_FOUND, "not found").into_response()
    }
}

#[derive(Deserialize, Debug)]
struct FormKey {
    key: String,
}

/// Show the form for set a key
async fn show_set_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/set" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

/// Set a key
async fn set_key(State(rpc_cli): State<RpcClient>, Form(setkey): Form<FormKey>) -> Response {
    rpc_cli.set(setkey.key.into()).await.unwrap();
    (StatusCode::OK, "set ok").into_response()
}

async fn show_del_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/del" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

async fn del_key(
    State(rpc_cli): State<RpcClient>,
    Form(delkey): Form<FormKey>,
) -> (StatusCode, &'static str) {
    rpc_cli.del(delkey.key.into()).await.unwrap();
    (StatusCode::OK, "del ok")
}
