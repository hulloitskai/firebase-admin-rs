use super::common::*;

use crate::api::IDENTITY_TOOLKIT_ENDPOINT;
use crate::app::App;

use builder::Builder;
use request::{Client, Response};

pub struct Authentication<'a> {
    app: &'a App,
    client: Client,
}

impl<'a> Authentication<'a> {
    pub(crate) fn new(app: &'a App) -> Self {
        Self {
            app,
            client: Client::new(),
        }
    }
}

impl<'a> Authentication<'a> {
    fn method_url(&self, name: &str) -> String {
        let project_id = self.app.project_id();
        format!(
            "{}/projects/{}/{}",
            IDENTITY_TOOLKIT_ENDPOINT, project_id, name
        )
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub created_at: DateTime,
    pub display_name: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Clone, Hash, Builder)]
#[builder(build_fn(private, name = "build_internal"))]
#[builder(setter(strip_option, into))]
pub struct UserConfig {
    id: String,

    #[builder(default)]
    display_name: Option<String>,

    email: String,

    #[builder(default)]
    email_verified: bool,

    #[builder(default)]
    phone_number: Option<String>,

    #[builder(default)]
    password: Option<String>,
}

impl UserConfig {
    pub fn builder() -> UserConfigBuilder {
        UserConfigBuilder::default()
    }
}

impl UserConfigBuilder {
    pub fn build(&mut self) -> Result<UserConfig> {
        self.build_internal().map_err(Error::msg)
    }
}

#[derive(Debug, Clone, Hash, Default, Builder)]
#[builder(build_fn(private, name = "build_internal"))]
#[builder(default)]
#[builder(setter(strip_option, into))]
pub struct UserUpdate {
    disable: Option<bool>,
    display_name: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    phone_number: Option<String>,
    password: Option<String>,

    #[builder(private)]
    delete_attribute: Vec<String>,

    #[builder(private)]
    delete_provider: Vec<String>,
}

impl UserUpdate {
    pub fn builder() -> UserUpdateBuilder {
        UserUpdateBuilder::default()
    }
}

impl UserUpdateBuilder {
    fn remove_attribute<'a>(&'a mut self, name: &str) -> &'a mut Self {
        let attributes = match &mut self.delete_attribute {
            Some(attributes) => attributes,
            None => {
                self.delete_attribute = Some(Vec::new());
                self.delete_attribute.as_mut().unwrap()
            }
        };
        attributes.push(name.to_owned());
        self
    }

    fn remove_provider<'a>(&'a mut self, name: &str) -> &'a mut Self {
        let providers = match &mut self.delete_provider {
            Some(providers) => providers,
            None => {
                self.delete_provider = Some(Vec::new());
                self.delete_provider.as_mut().unwrap()
            }
        };
        providers.push(name.to_owned());
        self
    }

    pub fn remove_display_name<'a>(&'a mut self) -> &'a mut Self {
        self.remove_attribute("DISPLAY_NAME")
    }

    pub fn remove_email<'a>(&'a mut self) -> &'a mut Self {
        self.remove_attribute("EMAIL")
    }

    pub fn remove_phone_number<'a>(&'a mut self) -> &'a mut Self {
        self.remove_provider("phone")
    }
}

impl UserUpdateBuilder {
    pub fn build(&mut self) -> Result<UserUpdate> {
        self.build_internal().map_err(Error::msg)
    }
}

impl<'a> Authentication<'a> {
    pub async fn get_user(&self, id: &str) -> Result<User> {
        let url = self.method_url("accounts:lookup");
        let body = json!({ "localId": [id] });
        let request = self.client.post(&url).json(&body);

        let response = self
            .app
            .send_request(request)
            .await
            .context("request failed")?;
        let response = handle_error_response(response).await?;
        let data: LookupAccountsResponse =
            response.json().await.context("failed to parse response")?;

        let users = data.users.context("not found")?;
        let user = users
            .into_iter()
            .next()
            .expect("empty users array in lookup response");
        user.try_into().context("failed to convert user")
    }

    pub async fn create_user(&self, config: UserConfig) -> Result<User> {
        let url = self.method_url("accounts");
        let body = RequestUserConfig::from(config);
        let request = self.client.post(&url).json(&body);

        let response = self
            .app
            .send_request(request)
            .await
            .context("request failed")?;
        let response = handle_error_response(response).await?;
        let data: CreateUserResponse =
            response.json().await.context("failed to parse response")?;

        let id = &data.local_id;
        self.get_user(id)
            .await
            .context("failed to get created user")
    }

    pub async fn update_user(
        &self,
        id: &str,
        update: UserUpdate,
    ) -> Result<User> {
        let url = self.method_url("accounts:update");
        let body = UpdateUserRequest {
            local_id: id.to_owned(),
            update: RequestUserUpdate::from(update),
        };
        let request = self.client.post(&url).json(&body);

        let response = self
            .app
            .send_request(request)
            .await
            .context("request failed")?;
        let response = handle_error_response(response).await?;
        let data: CreateUserResponse =
            response.json().await.context("failed to parse response")?;

        let id = &data.local_id;
        self.get_user(id)
            .await
            .context("failed to get updated user")
    }

    pub async fn delete_user(&self, id: &str) -> Result<()> {
        let url = self.method_url("accounts:delete");
        let body = json!({ "localId": id });
        let request = self.client.post(&url).json(&body);

        let response = self
            .app
            .send_request(request)
            .await
            .context("request failed")?;
        let _ = handle_error_response(response).await?;
        Ok(())
    }
}

pub async fn handle_error_response(
    response: Response,
) -> Result<Response, Error> {
    if response.status().is_success() {
        return Ok(response);
    }
    let data: ErrorResponse =
        response.json().await.context("failed to parse response")?;
    Err(Error::msg(data.error.message))
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: ResponseError,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseError {
    message: String,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LookupAccountsResponse {
    kind: String,
    users: Option<Vec<ResponseUser>>,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseUser {
    local_id: String,
    created_at: String,
    valid_since: String,
    display_name: Option<String>,
    email: String,
    email_verified: bool,
    phone_number: Option<String>,
    disabled: bool,
}

impl TryFrom<ResponseUser> for User {
    type Error = Error;

    fn try_from(user: ResponseUser) -> Result<Self, Self::Error> {
        let ResponseUser {
            local_id: id,
            created_at,
            display_name,
            email,
            email_verified,
            phone_number,
            disabled,
            ..
        } = user;
        let created_at: i64 = created_at
            .parse()
            .context("failed to parse 'created_at' as i64")?;
        let created_at = Utc.timestamp_millis(created_at);
        Ok(Self {
            id,
            created_at,
            display_name,
            email,
            email_verified,
            phone_number,
            disabled,
        })
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestUserConfig {
    pub local_id: String,
    pub display_name: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: Option<String>,
    pub password: Option<String>,
}

impl From<UserConfig> for RequestUserConfig {
    fn from(config: UserConfig) -> Self {
        let UserConfig {
            id: local_id,
            display_name,
            email,
            email_verified,
            phone_number,
            password,
        } = config;
        Self {
            local_id,
            display_name,
            email,
            email_verified,
            phone_number,
            password,
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUserResponse {
    local_id: String,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestUserUpdate {
    disable_user: Option<bool>,
    display_name: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    phone_number: Option<String>,
    password: Option<String>,
    delete_attribute: Option<Vec<String>>,
    delete_provider: Option<Vec<String>>,
}

impl From<UserUpdate> for RequestUserUpdate {
    fn from(update: UserUpdate) -> Self {
        let UserUpdate {
            disable: disable_user,
            display_name,
            email,
            email_verified,
            phone_number,
            password,
            delete_attribute,
            delete_provider,
        } = update;
        Self {
            disable_user,
            display_name,
            email,
            email_verified,
            phone_number,
            password,
            delete_attribute: if !delete_attribute.is_empty() {
                Some(delete_attribute)
            } else {
                None
            },
            delete_provider: if !delete_provider.is_empty() {
                Some(delete_provider)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateUserRequest {
    local_id: String,

    #[serde(flatten)]
    update: RequestUserUpdate,
}
