use anyhow::{ensure, Context, Result};
use yarrbot_common::environment;
use yarrbot_db::actions::user_actions::UserActions;
use yarrbot_db::enums::UserRole;
use yarrbot_db::models::{NewUser, User};
use yarrbot_db::{DbPool, DbPoolConnection};

const FIRST_USER_ENV: &str = "YARRBOT_INITIALIZATION_USER";

pub fn first_time_initialization(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;
    initialize_first_user(&conn)?;

    Ok(())
}

fn initialize_first_user(conn: &DbPoolConnection) -> Result<()> {
    debug!("Beginning initialization of the first user.");
    if User::any(conn)? {
        debug!("At least one user exists. Moving to the next step.");
        return Ok(());
    }

    let user_id_raw = environment::get_env_var(FIRST_USER_ENV).with_context(|| {
        format!(
            "Failed to retrieve a Matrix User ID from the {} environment variable.",
            FIRST_USER_ENV
        )
    })?;
    ensure!(
        yarrbot_matrix_client::is_user_id(&user_id_raw),
        "User ID provided is not valid."
    );

    let new_user = NewUser::new(&user_id_raw, Some(UserRole::SystemAdministrator));
    User::create_user(conn, new_user)
        .with_context(|| format!("Failed to create the User record for {}.", user_id_raw))?;

    info!("{} may now interact with Yarrbot.", user_id_raw);
    Ok(())
}
