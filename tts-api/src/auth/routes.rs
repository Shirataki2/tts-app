use crate::{auth::token::Token, error::AppError, AppState};
use actix_web::{get, http::header, web, HttpResponse};
use http::{HeaderMap, HeaderValue, Method};
use oauth2::{
    reqwest::http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse,
};
use sqlx::PgPool;
use url::Url;

#[derive(Deserialize, Debug)]
struct AuthRequestQuery {
    code: String,
    state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubUserData {
    pub id: i64,
}

#[get("/login")]
async fn login(data: web::Data<AppState>) -> HttpResponse {
    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = &data
        .oauth
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_code_challenge)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    HttpResponse::Found()
        .header(header::LOCATION, auth_url.to_string())
        .finish()
}

#[get("/auth")]
async fn auth(
    data: web::Data<AppState>,
    pool: web::Data<PgPool>,
    query: web::Query<AuthRequestQuery>,
) -> Result<HttpResponse, AppError> {
    let pool = pool.as_ref();

    let code = AuthorizationCode::new(query.code.clone());
    let _state = CsrfToken::new(query.state.clone());
    let token = &data
        .oauth
        .exchange_code(code)
        .request(http_client)
        .map_err(|_| AppError::RequestTokenError())?;
    let user_token = token.access_token();

    // TODO
    let url = Url::parse("https://api.github.com/user").unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(format!("Bearer {}", user_token.secret()).as_str())?,
    );
    let resp = http_client(oauth2::HttpRequest {
        url,
        method: Method::GET,
        headers,
        body: Vec::new(),
    })
    .map_err(|_| AppError::RequestTokenError())?;

    let user: GitHubUserData = serde_json::from_slice(&resp.body)?;

    eprintln!("{:?}", user);

    let token = Token::generate(24);
    token.register(pool, user.id).await?;

    let body = format!("Successfully Authorized!\n\nYour user id: {}\n\nYour user token is {}", user.id, token);

    Ok(HttpResponse::Ok().body(&body))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(auth);
}
