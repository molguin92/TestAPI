use std::net;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, Path, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, middleware, Router};
use clap::Parser;
use log::LevelFilter;
use reqwest::Url;
use serde_derive::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use tokio::time::Instant;

use test_api::{APITask, ApiError, API};

async fn request_logger(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let recv_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    log::info!(
        "{} {} from {} - {} (latency: {} ms)",
        method,
        uri,
        addr,
        response.status(),
        (Instant::now() - recv_time).as_millis()
    );
    response
}

/// TestAPI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to bind to and serve from
    #[arg(short = 'b', long, default_value_t = Url::parse("http://0.0.0.0:8080/api").unwrap())]
    bind_addr: Url,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);

    let bind_host = format!(
        "{}:{}",
        args.bind_addr.host_str().unwrap(),
        args.bind_addr.port_or_known_default().unwrap()
    );

    let sock_addr = match tokio::net::lookup_host(bind_host).await {
        Ok(mut s) => match s.next() {
            Some(s) => s,
            None => unreachable!(),
        },
        Err(e) => {
            panic!("Cannot bind to requested address: {}", e)
        }
    };

    let api = API::new();

    let app = Router::new()
        .route("/api/tasks", get(get_task_handler))
        .route("/api/tasks/:task_id", post(post_task_handler))
        .layer(middleware::from_fn(request_logger))
        .with_state(api);

    let listener = tokio::net::TcpListener::bind(sock_addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<net::SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn get_task_handler(
    State(api): State<Arc<API>>,
) -> Result<(StatusCode, Json<APITask>), (StatusCode, impl IntoResponse)> {
    api.new_task()
        .await
        .map(|t| (StatusCode::OK, Json(t.clone())))
        .map_err(|e| match e {
            ApiError::APIError(s) => (StatusCode::INTERNAL_SERVER_ERROR, ()),
            ApiError::NoSuchTask => unreachable!(),
            ApiError::IncorrectResult(_) => unreachable!(),
            ApiError::AuthError => (StatusCode::UNAUTHORIZED, ()),
        })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReqBody {
    result: i8,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ResultBody {
    success: bool,
    error: Option<String>,
    received: Option<i8>,
    expected: Option<i8>,
}

async fn post_task_handler(
    State(api): State<Arc<API>>,
    Path(task_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<ReqBody>,
) -> Result<(StatusCode, Json<ResultBody>), (StatusCode, Json<ResultBody>)> {
    let bearer_token: String = match headers.get(header::AUTHORIZATION) {
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ResultBody {
                    success: false,
                    error: Some("must provide an Authorization header with a valid token".into()),
                    received: None,
                    expected: None,
                }),
            ))
        }
        Some(h) => h
            .to_str()
            .map_err(|e| {
                (
                    StatusCode::FORBIDDEN,
                    Json(ResultBody {
                        success: false,
                        error: Some(
                            "must provide an Authorization header with a valid token".into(),
                        ),
                        received: None,
                        expected: None,
                    }),
                )
            })?
            .into(),
    };

    let token: String = match bearer_token.split_whitespace().last() {
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ResultBody {
                    success: false,
                    error: Some("must provide an Authorization header with a valid token".into()),
                    received: None,
                    expected: None,
                }),
            ))
        }
        Some(t) => t.into(),
    };

    api.validate_result(task_id, token, payload.result.clone())
        .await
        .map(|_| {
            (
                StatusCode::OK,
                Json(ResultBody {
                    success: true,
                    error: None,
                    received: Some(payload.result.clone()),
                    expected: Some(payload.result.clone()),
                }),
            )
        })
        .map_err(|e| match e {
            ApiError::AuthError => (
                StatusCode::UNAUTHORIZED,
                Json(ResultBody {
                    success: false,
                    error: Some("unauthorized".to_string()),
                    received: None,
                    expected: None,
                }),
            ),
            ApiError::APIError(s) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultBody {
                    success: false,
                    error: Some(s),
                    received: None,
                    expected: None,
                }),
            ),
            ApiError::NoSuchTask => (
                StatusCode::NOT_FOUND,
                Json(ResultBody {
                    success: false,
                    error: Some("task id not found".to_string()),
                    received: None,
                    expected: None,
                }),
            ),
            ApiError::IncorrectResult(expected) => (
                StatusCode::UNAUTHORIZED,
                Json(ResultBody {
                    success: false,
                    error: Some("incorrect result".to_string()),
                    received: Some(payload.result.clone()),
                    expected: Some(expected),
                }),
            ),
        })
}
