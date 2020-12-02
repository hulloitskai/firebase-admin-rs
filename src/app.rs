use super::common::*;

use crate::api::OAUTH_SCOPES;
use crate::auth::Auth;

use gcloud_auth::init as init_auth_manager;
use gcloud_auth::AuthenticationManager as AuthManager;
use gcloud_auth::Error as AuthError;
use gcloud_auth::Token as AuthToken;

use request::{Client, IntoUrl, Method, RequestBuilder};
use tokio::runtime::Builder as RuntimeBuilder;

pub struct App {
    project_id: String,
    client: Client,
    auth: AuthManager,
}

impl App {
    pub fn new(project_id: &str) -> Result<Self> {
        let runtime = RuntimeBuilder::new_current_thread()
            .build()
            .context("failed to initialize Tokio runtime")?;
        let auth_manager = runtime
            .block_on(init_auth_manager().compat())
            .context("failed to initialize authentication manager")?;
        let app = Self {
            project_id: project_id.to_owned(),
            client: Client::new(),
            auth: auth_manager,
        };
        Ok(app)
    }

    pub fn auth<'a>(&'a self) -> Auth<'a> {
        Auth::new(self)
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

    pub(super) async fn request<U: IntoUrl>(
        &self,
        method: Method,
        url: U,
    ) -> Result<RequestBuilder> {
        let token = self
            .get_token()
            .compat()
            .await
            .context("failed to get authentication token")?;
        let request =
            self.client.request(method, url).bearer_auth(token.as_str());
        Ok(request)
    }
}
