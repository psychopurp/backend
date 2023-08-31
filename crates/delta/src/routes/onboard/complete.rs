use once_cell::sync::Lazy;
use regex::Regex;
use revolt_database::{Channel, Database, User};
use revolt_models::v0;
use revolt_quark::{authifier::models::Session, variables::delta::OFFICIAL_MODEL_BOTS};
use revolt_result::{create_error, Result};

use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    /// New username which will be used to identify the user on the platform
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

/// # Complete Onboarding
///
/// This sets a new username, completes onboarding and allows a user to start using Revolt.
#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<Json<v0::User>> {
    if user.is_some() {
        return Err(create_error!(AlreadyOnboarded));
    }

    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let new_user = User::create(db, data.username, session.user_id, None).await?;

    prepare_on_board_data(db, new_user.id.clone()).await?;

    Ok(Json(new_user.into_self().await))
}

/// prepare on board data for the first time login
async fn prepare_on_board_data(db: &Database, user_id: String) -> Result<()> {
    if (*OFFICIAL_MODEL_BOTS).is_empty() {
        return Ok(());
    }

    let id = Ulid::new().to_string();
    let users = vec![user_id.clone()];

    let mut group = Channel::Group {
        id,
        name: String::from("多模型群聊"),
        owner: user_id.clone(),
        description: Some(String::from("默认群聊，可以通过@来调用大模型")),
        recipients: users,
        icon: None,
        last_message_id: None,
        permissions: None,
        nsfw: false,
    };

    group.create(db).await?;
    let bot_users = db.fetch_users(OFFICIAL_MODEL_BOTS.as_slice()).await?;
    for bot in bot_users {
        group.add_user_to_group(&db.clone(), &bot, &user_id).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{rocket, routes::onboard::complete::DataOnboard, util::test::TestHarness};
    use revolt_quark::variables::delta::OFFICIAL_MODEL_BOTS;
    use rocket::http::{ContentType, Header, Status};

    #[rocket::async_test]
    async fn test_on_board_compelete() {
        let harness = TestHarness::new().await;
        let (_, session) = harness.new_account_session().await;

        let mut users = vec![];
        users.extend((*OFFICIAL_MODEL_BOTS).clone());
        for user_bot in users {
            // let _ = harness.db.delete_user(&user_bot).await;
            // let _ = User::create(&harness.db, user_bot.clone(), Some(user_bot.clone()), None).await;
            println!("{user_bot}");
        }

        let response = harness
            .client
            .post("/onboard/complete")
            .header(Header::new("x-session-token", session.token.to_string()))
            .header(ContentType::JSON)
            .body(
                json!(DataOnboard {
                    username: "cac".into()
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        // println!("{:}", response.into_string().await.unwrap());
    }
}
