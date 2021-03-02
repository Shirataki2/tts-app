pub mod routes;
pub mod token;

pub use self::routes::init;
pub use self::routes::GitHubUserData;

use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::env;

pub fn create_auth_client() -> BasicClient {
    let client_id =
        ClientId::new(env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID is not set"));
    let client_secret = ClientSecret::new(
        env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET is not set"),
    );
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid AuthUrl");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid TokenUrl");
    let redirect_url = RedirectUrl::new(env::var("REDIRECT_URL").expect("REDIRECT_URL is not set"))
        .expect("Invalid RedirectUrl");

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_url(redirect_url)
}
