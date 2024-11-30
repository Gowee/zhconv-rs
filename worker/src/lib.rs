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

use std::{cell::OnceCell, env};

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
    // let target = Variant::from_str(&target).expect("Unsupported target variant");

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
    let mut info = Vec::new();

    if let Some(version) = option_env!("CARGO_PKG_VERSION") {
        info.push(format!("Version: {}", version));
    }
    // Build Information
    if let Some(timestamp) = option_env!("VERGEN_BUILD_TIMESTAMP") {
        info.push(format!("Built on: {}", timestamp));
    }
    if let Some(date) = option_env!("VERGEN_BUILD_DATE") {
        info.push(format!("Build date: {}", date));
    }
    if let Some(debug) = option_env!("VERGEN_CARGO_DEBUG") {
        info.push(format!("Debug build: {}", debug));
    }
    if let Some(features) = option_env!("VERGEN_CARGO_FEATURES") {
        info.push(format!("Features: {}", features));
    }
    if let Some(target) = option_env!("VERGEN_CARGO_TARGET_TRIPLE") {
        info.push(format!("Target: {}", target));
    }
    if let Some(rustc) = option_env!("VERGEN_RUSTC_VERSION") {
        info.push(format!("Rust version: {}", rustc));
    }

    // Git Information
    if let Some(branch) = option_env!("VERGEN_GIT_BRANCH") {
        info.push(format!("Branch: {}", branch));
    }
    if let Some(commit_date) = option_env!("VERGEN_GIT_COMMIT_DATE") {
        info.push(format!("Commit date: {}", commit_date));
    }
    if let Some(sha) = option_env!("VERGEN_GIT_SHA") {
        info.push(format!("Commit: {}", sha));
    }
    if let Some(describe) = option_env!("VERGEN_GIT_DESCRIBE") {
        info.push(format!("Git describe: {}", describe));
    }
    if let Some(msg) = option_env!("VERGEN_GIT_COMMIT_MESSAGE") {
        info.push(format!("Commit message: {}", msg));
    }
    if let Some(commit) = option_env!("MEDIAWIKI_COMMIT_HASH") {
        info.push(format!("MediaWiki commit: {}", commit));
    }
    if let Some(commit) = option_env!("OPENCC_COMMIT_HASH") {
        info.push(format!("OpenCC commit: {}", commit));
    }

    // Join all available info with newlines
    info.join("\n")
}
