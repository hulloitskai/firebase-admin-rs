use dotenv::dotenv;
use dotenv::Error as DotenvError;

use std::env::var as get_env_var;
use std::io::ErrorKind as IoErrorKind;

use anyhow::{Context as ResultContext, Result};
use tokio::runtime::Runtime;
use uuid::Uuid;

use firebase_admin::{App, UserConfig, UserUpdate};

fn main() -> Result<()> {
    if let Err(DotenvError::Io(error)) = dotenv() {
        if error.kind() != IoErrorKind::NotFound {
            return Err(error).context("failed to load .env");
        }
    }

    let project_id = get_env_var("FIREBASE_PROJECT_ID")
        .context("failed to get FIREBASE_PROJECT_ID")?;
    let app = App::new(project_id).context("failed to initialize app")?;
    let auth = app.authentication();

    let runtime = Runtime::new().context("initialize Tokio runtime")?;
    runtime.block_on(async {
        let user_config = UserConfig::builder()
            .id(Uuid::new_v4().to_string())
            .display_name("Jon Snow")
            .email("jon.snow@example.com")
            .phone_number("+1 (234) 567-8910")
            .build()
            .context("failed to build user config")?;
        let user = auth
            .create_user(user_config)
            .await
            .context("failed to create user")?;
        println!("Created a user: {:#?}", &user);
        let user_id = &user.id;

        let user_update = UserUpdate::builder()
            .email_verified(true)
            .remove_phone_number()
            .build()
            .context("failed to build user update")?;
        let user = auth
            .update_user(user_id, user_update)
            .await
            .context("failed to update user")?;
        println!("Updated user: {:#?}", &user);

        auth.delete_user(user_id)
            .await
            .context("failed to delete user")?;
        println!("User {} was deleted", user_id);
        Ok(())
    })
}
