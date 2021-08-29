use anyhow::Result;
use tokio::task::spawn_blocking;
use yarrbot_db::actions::user_actions::UserActions;
use yarrbot_db::models::User;
use yarrbot_db::DbPool;

mod add;
mod list;
mod remove;

pub use add::handle_add;
pub use list::handle_list;
pub use remove::handle_remove;

async fn get_user(pool: &DbPool, username: &str) -> Result<Option<User>> {
    let conn = pool.get()?;
    let username2 = String::from(username);
    Ok(spawn_blocking(move || User::try_get_by_username(&conn, &username2)).await??)
}
