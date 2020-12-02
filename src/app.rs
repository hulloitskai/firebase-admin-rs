use super::common::*;

use crate::api::OAUTH_SCOPES;
use crate::auth::Authentication;

use gcloud_auth::init as init_auth_manager;
use gcloud_auth::AuthenticationManager as AuthManager;
use gcloud_auth::Error as AuthError;
use gcloud_auth::Token as AuthToken;

use request::{RequestBuilder, Response};
use tokio::runtime::Builder as RuntimeBuilder;

pub struct App {
    project_id: String,
    auth: AuthManager,
}

impl App {
    pub fn new(project_id: String) -> Result<Self> {
        let runtime = RuntimeBuilder::new_current_thread()
            .build()
            .context("failed to initialize Tokio runtime")?;
        let auth_manager = runtime
            .block_on(init_auth_manager().compat())
            .context("failed to initialize authentication manager")?;
        let app = Self {
            project_id,
            auth: auth_manager,
        };
        Ok(app)
    }

    pub fn authentication<'a>(&'a self) -> Authentication<'a> {
        Authentication::new(self)
    }
}

impl App {
    pub(crate) fn project_id<'a>(&'a self) -> &'a str {
        &self.project_id
    }
}

impl App {
    async fn get_token(&self) -> Result<AuthToken, AuthError> {
        self.auth.get_token(&OAUTH_SCOPES).await
    }

    pub(crate) async fn send_request(
        &self,
        request: RequestBuilder,
    ) -> Result<Response> {
        let token = self
            .get_token()
            .compat()
            .await
            .context("failed to get authentication token")?;
        let response =
            request.bearer_auth(token.as_str()).send().compat().await?;
        Ok(response)
    }
}
