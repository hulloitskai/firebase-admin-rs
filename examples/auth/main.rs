use dotenv::dotenv;
use dotenv::Error as DotenvError;

use std::env::var as get_env_var;
use std::io::ErrorKind as IoErrorKind;

use anyhow::{Context as ResultContext, Result};
use tokio::runtime::Runtime;
use uuid::Uuid;

use firebase_admin::{
    App, Auth, CreateUserParams, UpdateUserParams, UserRecord,
};

fn main() -> Result<()> {
    if let Err(DotenvError::Io(error)) = dotenv() {
        if error.kind() != IoErrorKind::NotFound {
            return Err(error).context("failed to load .env");
        }
    }

    let project_id = get_env_var("FIREBASE_PROJECT_ID")
        .context("failed to get FIREBASE_PROJECT_ID")?;
    let app = App::new(&project_id).context("failed to initialize app")?;
    let auth = app.auth();

    let runtime = Runtime::new().context("initialize Tokio runtime")?;
    runtime.block_on(async {
        let user = create_user(&auth).await.context("failed to create user")?;
        let uid = &user.uid;
        println!("Created a user: {:#?}", &user);

        let user = update_user(&auth, uid)
            .await
            .context("failed to update user")?;
        println!("Updated user: {:#?}", &user);

        auth.delete_user(uid)
            .await
            .context("failed to delete user")?;
        println!("Deleted user with UID: {}", uid);

        Ok(())
    })
}

async fn create_user(auth: &Auth<'_>) -> Result<UserRecord> {
    let params = CreateUserParams::builder()
        .uid(Uuid::new_v4().to_string())
        .display_name("Jon Snow")
        .email("jon.snow@example.com")
        .phone_number("+1 (234) 567-8910")
        .build()
        .context("invalid params")?;
    auth.create_user(params).await
}

async fn update_user(auth: &Auth<'_>, uid: &str) -> Result<UserRecord> {
    let params = UpdateUserParams::builder()
        .email_verified(true)
        .remove_phone_number()
        .build()
        .context("invalid params")?;
    auth.update_user(uid, params).await
}
