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

use std::env;

const DOC: &str = include_str!("../doc.txt");

fn router() -> Router {
    Router::new()
        .route("/", get(doc))
        .route("/convert/:target", post(convert))
        .route("/is-hans", post(is_hans))
        .route("/version", get(version))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router().call(req).await?)
}

pub async fn doc() -> &'static str {
    DOC
}

#[derive(Deserialize)]
pub struct ConvertQuery {
    // #[serde(default = false)]
    wikitext: Option<bool>,
}

pub async fn convert(
    Path(target): Path<Variant>,
    Query(params): Query<ConvertQuery>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    if let Ok(token) = env::var("TOKEN") {
        if bearer.as_ref().map(|b| b.token()) == Some(&token) {
            return (
                StatusCode::UNAUTHORIZED,
                String::from("Unauthorized - Token is set by the TOKEN envvar"),
            );
        }
    }
    let wikitext = params.wikitext.unwrap_or(false);

    let response_body = if wikitext {
        zhconv_mw(&body, target)
    } else {
        zhconv_plain(&body, target)
    };

    (StatusCode::OK, response_body)
}

pub async fn is_hans(body: String) -> impl IntoResponse {
    is_hans_confidence(&body).to_string()
}

pub async fn version() -> impl IntoResponse {
    option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN")
}
