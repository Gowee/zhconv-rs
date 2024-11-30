use zhconv::{is_hans_confidence, zhconv as zhconv_plain, zhconv_mw, Variant};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, Router},
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::Deserialize;
use tower_service::Service;
use worker::*;

use std::sync::OnceLock;

mod utils;
use utils::bool_from_int;

const DOC: &str = include_str!("../doc.txt");
static API_TOKEN: OnceLock<Option<String>> = OnceLock::new();

fn router() -> Router {
    Router::new()
        .route("/", get(doc))
        .route("/convert/:target", post(convert))
        .route("/is-hans", post(is_hans))
        .route("/version", get(version))
        .fallback(handle_404)
        .method_not_allowed_fallback(handle_405)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    let _ = API_TOKEN.get_or_init(|| env.secret("API_TOKEN").ok().map(|t| t.to_string()));
    Ok(router().call(req).await?)
}

pub async fn doc() -> &'static str {
    DOC
}

#[derive(Deserialize)]
pub struct ConvertQuery {
    // #[serde(default = false)]
    #[serde(default, deserialize_with = "bool_from_int")]
    wikitext: bool,
}

pub async fn convert(
    Path(target): Path<Variant>,
    Query(params): Query<ConvertQuery>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    ensure_authorized!(bearer);
    let wikitext = params.wikitext;

    let response_body = if wikitext {
        zhconv_mw(&body, target)
    } else {
        zhconv_plain(&body, target)
    };

    (StatusCode::OK, response_body)
}

pub async fn is_hans(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    ensure_authorized!(bearer);

    (StatusCode::OK, is_hans_confidence(&body).to_string())
}

pub async fn version() -> impl IntoResponse {
    option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN")
}

pub async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not found")
}

pub async fn handle_405() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method not allowed")
}

macro_rules! ensure_authorized {
    ($bearer:expr) => {
        if let Some(token) = API_TOKEN.get().map(|t| t.as_ref()).flatten() {
            if $bearer.as_ref().map(|b| b.token()) != Some(&token) {
                return (
                    StatusCode::UNAUTHORIZED,
                    String::from("401 Unauthorized - Token is set by the API_TOKEN envvar"),
                );
            }
        }
    };
}
use ensure_authorized;
