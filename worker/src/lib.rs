#[cfg(all(
    any(feature = "mediawiki-hans", feature = "opencc-hans"),
    any(feature = "mediawiki-hant", feature = "opencc-hant")
))]
use zhconv::is_hans_confidence;
use zhconv::{zhconv as zhconv_plain, zhconv_mw, Variant, ENABLED_TARGET_VARIANTS};

use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, Router},
};
use axum_extra::{response::ErasedJson, TypedHeader};
use headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use tower_service::Service;
use worker::*;

mod utils;
use utils::bool_from_int;

const DOC: &str = include_str!("../doc.txt");
const DEFAULT_BODY_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone, Default, Debug)]
pub struct AppState {
    api_token: Option<String>,
    body_limit: Option<usize>,
}

fn router(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(doc))
        .route("/convert/{target}", post(convert))
        .route("/info", get(info))
        .layer(DefaultBodyLimit::max(
            state.body_limit.unwrap_or(DEFAULT_BODY_LIMIT),
        ))
        .fallback(handle_404)
        .method_not_allowed_fallback(handle_405)
        .with_state(state);
    #[cfg(all(
        any(feature = "mediawiki-hans", feature = "opencc-hans"),
        any(feature = "mediawiki-hant", feature = "opencc-hant")
    ))]
    let router = router.route("/is-hans", post(is_hans));
    router
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    let state = AppState {
        api_token: env.secret("API_TOKEN").ok().map(|t| t.to_string()),
        body_limit: env
            .var("BODY_LIMIT")
            .ok()
            .and_then(|v| v.to_string().parse().ok()),
    };
    Ok(router(state).call(req).await?)
}

pub async fn doc() -> &'static str {
    DOC
}

#[derive(Deserialize)]
pub struct ConvertQuery {
    #[serde(default, deserialize_with = "bool_from_int")]
    wikitext: bool,
}

pub async fn convert(
    State(state): State<AppState>,
    Path(target): Path<Variant>,
    Query(params): Query<ConvertQuery>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    ensure_authorized!(state, bearer);
    if !ENABLED_TARGET_VARIANTS.contains(&target) {
        return (
            StatusCode::BAD_REQUEST,
            String::from("400 Target variant not enabled"),
        );
    }
    let wikitext = params.wikitext;

    let response_body = if wikitext {
        zhconv_mw(&body, target)
    } else {
        zhconv_plain(&body, target)
    };

    (StatusCode::OK, response_body)
}
#[cfg(all(
    any(feature = "mediawiki-hans", feature = "opencc-hans"),
    any(feature = "mediawiki-hant", feature = "opencc-hant")
))]
pub async fn is_hans(
    State(state): State<AppState>,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    body: String,
) -> impl IntoResponse {
    ensure_authorized!(state, bearer);

    (StatusCode::OK, is_hans_confidence(&body).to_string())
}

#[derive(Serialize)]
pub struct Info {
    version: &'static str,
    auth_enabled: bool,
    body_limit: usize,
    enabled_target_variants: &'static [Variant],
}

pub async fn info(State(state): State<AppState>) -> impl IntoResponse {
    ErasedJson::pretty(Info {
        version: option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN"),
        auth_enabled: state.api_token.is_some(),
        body_limit: state.body_limit.unwrap_or(DEFAULT_BODY_LIMIT),
        enabled_target_variants: ENABLED_TARGET_VARIANTS,
    })
}

pub async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not found")
}

pub async fn handle_405() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method not allowed")
}

macro_rules! ensure_authorized {
    ($state:expr, $bearer:expr) => {
        if let Some(token) = $state.api_token {
            if $bearer.as_ref().map(|b| b.token()) != Some(&token) {
                return (StatusCode::UNAUTHORIZED, String::from("401 Unauthorized"));
            }
        }
    };
}
use ensure_authorized;
