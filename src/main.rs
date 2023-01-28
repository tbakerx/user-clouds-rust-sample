use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenType,
};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    ExtraTokenFields, RedirectUrl, Scope, StandardRevocableToken, StandardTokenResponse,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// These are test credentials for the sample app.
///
/// You should replace these with your own.
const USER_CLOUDS_URL: &str = "https://sample.tenant.userclouds.com";
const USER_CLOUDS_CLIENT_ID: &str = "5f107e226353791560f93164a09f7e0f";
const USER_CLOUDS_CLIENT_SECRET: &str =
    "2ftQe4RU7aR/iStpcFf3gfiUjnsbWGFY0C9aWkPzDqT14eyp23ysKuI6iBbOAW/O";
const USER_CLOUDS_REDIRECT_URL: &str = "http://localhost:3000/callback";

struct AppState {
    oauth2_client: Client<
        BasicErrorResponse,
        StandardTokenResponse<ExtraFields, BasicTokenType>,
        BasicTokenType,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
    >,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        oauth2_client: Client::new(
            ClientId::new(USER_CLOUDS_CLIENT_ID.to_string()),
            Some(ClientSecret::new(USER_CLOUDS_CLIENT_SECRET.to_string())),
            AuthUrl::new(format!("{}/oidc/authorize", USER_CLOUDS_URL)).unwrap(),
            Some(TokenUrl::new(format!("{}/oidc/token", USER_CLOUDS_URL)).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(USER_CLOUDS_REDIRECT_URL.to_string()).unwrap()),
    });

    let app = Router::new()
        .route("/", get(health))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .with_state(shared_state);

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), 3000);
    println!("server listening on http://{} ...", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn login(State(app_state): State<Arc<AppState>>) -> Response {
    // Generate the authorization URL.
    let (auth_url, _) = app_state
        .oauth2_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Redirect the user to the authorization URL.
    Redirect::to(&auth_url.to_string()).into_response()
}

async fn callback(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<AuthRequest>,
) -> Json<Value> {
    let code = AuthorizationCode::new(params.code.clone());
    let state = CsrfToken::new(params.state.clone());

    // Exchange the code with a token.
    let token = app_state
        .oauth2_client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
        .unwrap();

    let scopes = if let Some(scopes_vec) = token.scopes() {
        scopes_vec
            .iter()
            .flat_map(|comma_separated| comma_separated.split(','))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let auth_response = AuthResponse {
        access_token: token.access_token(),
        token_type: token.token_type(),
        id_token: &token.extra_fields().id_token,
        secret: state.secret(),
        scopes,
    };

    Json(json!(auth_response))
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtraFields {
    id_token: String,
}
impl ExtraTokenFields for ExtraFields {}

#[derive(Serialize)]
pub struct AuthResponse<'a> {
    access_token: &'a AccessToken,
    token_type: &'a BasicTokenType,
    id_token: &'a String,
    secret: &'a String,
    scopes: Vec<&'a str>,
}
