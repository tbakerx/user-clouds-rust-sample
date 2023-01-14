use async_trait::async_trait;

mod error;
use error::Error;
use mendes::http::request::Parts;
use mendes::http::{Response, StatusCode};
use mendes::hyper::Body;
use mendes::{handler, route, Application, Context};
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

pub struct App {
    #[allow(dead_code)]
    oauth2_client: Client<
        BasicErrorResponse,
        StandardTokenResponse<ExtraFields, BasicTokenType>,
        BasicTokenType,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
    >,
}

impl App {
    pub async fn new() -> anyhow::Result<Self> {
        let oauth2_client = Client::new(
            ClientId::new(USER_CLOUDS_CLIENT_ID.to_string()),
            Some(ClientSecret::new(USER_CLOUDS_CLIENT_SECRET.to_string())),
            AuthUrl::new(format!("{}/oidc/authorize", USER_CLOUDS_URL))?,
            Some(TokenUrl::new(format!("{}/oidc/token", USER_CLOUDS_URL))?),
        )
        .set_redirect_uri(RedirectUrl::new(
            USER_CLOUDS_REDIRECT_URL.to_string()
        )?);

        Ok(Self { oauth2_client })
    }
}

/// These are test credentials for the sample app.
/// 
/// You should replace these with your own.
const USER_CLOUDS_URL: &str = "https://sample.tenant.userclouds.com";
const USER_CLOUDS_CLIENT_ID: &str = "5f107e226353791560f93164a09f7e0f";
const USER_CLOUDS_CLIENT_SECRET: &str =
    "2ftQe4RU7aR/iStpcFf3gfiUjnsbWGFY0C9aWkPzDqT14eyp23ysKuI6iBbOAW/O";
const USER_CLOUDS_REDIRECT_URL: &str = "http://localhost:8080/callback";

#[async_trait]
impl Application for App {
    type RequestBody = Body;
    type ResponseBody = Body;
    type Error = Error;

    async fn handle(mut cx: Context<Self>) -> Response<Body> {
        route!(match cx.path() {
            Some("callback") => callback,
            Some("login") => login,
            Some("logout") => logout,
            None => health,
        })
    }
}

#[handler(GET)]
async fn health(_: &App) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("{status: ok}".into())
        .unwrap())
}

#[handler(GET)]
async fn callback(
    app: &App,
    _: &Parts,
    #[query] query: AuthRequest,
) -> Result<Response<Body>, Error> {
    let code = AuthorizationCode::new(query.code.clone());
    let state = CsrfToken::new(query.state.clone());

    // Exchange the code with a token.
    let token = app
        .oauth2_client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
        .map_err(|err| Error::internal(format!("oauth2 error: {:?}", err)))?;

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

    let body_string = serde_json::to_string(&auth_response)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(body_string.into())
        .unwrap())
}

#[handler(GET)]
async fn login(app: &App) -> Result<Response<Body>, Error> {
    // Generate the authorization URL.
    let (auth_url, _) = app
        .oauth2_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(App::redirect(StatusCode::TEMPORARY_REDIRECT, auth_url))
}

#[handler(GET)]
async fn logout(_: &App) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("logout".into())
        .unwrap())
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
