use crate::util::header::Headers;
use reqwest::header::COOKIE;
use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_quark::authifier::{models::Session, Authifier};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket::{http::CookieJar, serde::json::Json};
use serde::{Deserialize, Serialize};

/// # Login Data
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataLogin {
    /// Friendly name used for the session
    pub friendly_name: Option<String>,
}

/// # Login
///
/// Login to an account.
#[openapi(tag = "Session")]
#[post("/login", data = "<data>")]
pub async fn login(
    db: &State<Database>,
    authifier: &State<Authifier>,
    data: Json<DataLogin>,
    cookies: &CookieJar<'_>,
    headers: Headers<'_>,
) -> Result<Json<Session>> {
    let data = data.into_inner();

    let mut cookie_str = String::new();
    for cookie in cookies.iter() {
        cookie_str.push_str(&format!("{}={}; ", cookie.name(), cookie.value()));
    }

    let config = revolt_config::config().await;
    let service_url = headers.0.get("Referer").next().unwrap_or("");
    let url = format!(
        "{}/v1/login?service={}",
        config.api.botservice.chatall_server, service_url
    );

    let client = reqwest::Client::new();
    let response: v0::UUAPResponse = client
        .get(url.clone())
        .header(COOKIE, cookie_str)
        .send()
        .await
        .expect("SendRequestFailed")
        .json()
        .await
        .expect("ParseJsonFailed");

    match &response.data {
        v0::UUAPResponseData::Forbidden { username } => Err(create_error!(ForbiddenUser {
            username: username.clone()
        })),
        v0::UUAPResponseData::Redirect(uri) => {
            Err(create_error!(LoginRedirect { uri: uri.clone() }))
        }

        v0::UUAPResponseData::Success { user, .. } => {
            let (account, _user) = User::get_or_create_new_user(
                authifier,
                db,
                user.username.clone(),
                user.email.clone(),
            )
            .await?;

            let session = account
                .create_session(authifier, data.friendly_name.unwrap_or("".into()))
                .await
                .unwrap();

            Ok(Json(session))
        }
    }
}
