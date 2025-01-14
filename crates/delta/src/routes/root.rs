use revolt_quark::variables::delta::{
    HCAPTCHA_SITEKEY, INVITE_ONLY, USE_EMAIL, USE_HCAPTCHA, USE_VOSO, VAPID_PUBLIC_KEY, VOSO_URL,
    VOSO_WS_HOST,
};
use revolt_quark::Result;

use rocket::serde::json::Json;
use serde::Serialize;

/// # hCaptcha Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct CaptchaFeature {
    /// Whether captcha is enabled
    pub enabled: bool,
    /// Client key used for solving captcha
    pub key: String,
}

/// # Generic Service Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct Feature {
    /// Whether the service is enabled
    pub enabled: bool,
    /// URL pointing to the service
    pub url: String,
}

/// # Voice Server Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct VoiceFeature {
    /// Whether voice is enabled
    pub enabled: bool,
    /// URL pointing to the voice API
    pub url: String,
    /// URL pointing to the voice WebSocket server
    pub ws: String,
}

/// # Feature Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltFeatures {
    /// hCaptcha configuration
    pub captcha: CaptchaFeature,
    /// Whether email verification is enabled
    pub email: bool,
    /// Whether this server is invite only
    pub invite_only: bool,
    /// File server service configuration
    pub autumn: Feature,
    /// Proxy service configuration
    pub january: Feature,
    /// Voice server configuration
    pub voso: VoiceFeature,
    /// Prompt server service configuration
    pub promptserv: Feature,
}

/// # Build Information
#[derive(Serialize, JsonSchema, Debug)]
pub struct BuildInformation {
    /// Commit Hash
    pub commit_sha: String,
    /// Commit Timestamp
    pub commit_timestamp: String,
    /// Git Semver
    pub semver: String,
    /// Git Origin URL
    pub origin_url: String,
    /// Build Timestamp
    pub timestamp: String,
}

/// # Server Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltConfig {
    /// Revolt API Version
    pub revolt: String,
    /// Features enabled on this Revolt node
    pub features: RevoltFeatures,
    /// WebSocket URL
    pub ws: String,
    /// URL pointing to the client serving this node
    pub app: String,
    /// Web Push VAPID public key
    pub vapid: String,
    /// Build information
    pub build: BuildInformation,
}

/// # Query Node
///
/// Fetch the server configuration for this Revolt instance.
#[openapi(tag = "Core")]
#[get("/")]
pub async fn root() -> Result<Json<RevoltConfig>> {
    let config = revolt_config::config().await;

    Ok(Json(RevoltConfig {
        revolt: env!("CARGO_PKG_VERSION").to_string(),
        features: RevoltFeatures {
            captcha: CaptchaFeature {
                enabled: *USE_HCAPTCHA,
                key: HCAPTCHA_SITEKEY.to_string(),
            },
            email: *USE_EMAIL,
            invite_only: *INVITE_ONLY,
            autumn: Feature {
                enabled: !config.hosts.autumn.is_empty(),
                url: config.hosts.autumn,
            },
            january: Feature {
                enabled: !config.hosts.january.is_empty(),
                url: config.hosts.january,
            },
            promptserv: Feature {
                enabled: !config.hosts.promptserv.is_empty(),
                url: config.hosts.promptserv,
            },
            voso: VoiceFeature {
                enabled: *USE_VOSO,
                url: VOSO_URL.to_string(),
                ws: VOSO_WS_HOST.to_string(),
            },
        },
        ws: config.hosts.events,
        app: config.hosts.app,
        vapid: VAPID_PUBLIC_KEY.to_string(),
        build: BuildInformation {
            commit_sha: option_env!("VERGEN_GIT_SHA")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            commit_timestamp: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            semver: option_env!("VERGEN_GIT_SEMVER")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            origin_url: option_env!("GIT_ORIGIN_URL")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            timestamp: option_env!("VERGEN_BUILD_TIMESTAMP")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
        },
    }))
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use rocket::http::Status;

    #[rocket::async_test]
    async fn hello_world() {
        let harness = crate::util::test::TestHarness::new().await;
        let response = harness.client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn hello_world_concurrent() {
        let harness = crate::util::test::TestHarness::new().await;
        let response = harness.client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}
