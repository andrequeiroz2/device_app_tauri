use sqlx::{Pool, Sqlite};
use crate::api::user::user_model::{CreateUserInput, UserResponseDB};
use crate::api::user::user_query;
use tracing::{instrument, info, error};

#[instrument(skip(user, pool), fields(username = %user.username, email = %user.email, password = %user.password, confirm_password = %user.confirm_password))]
pub async fn user_create_handler(
    user: &CreateUserInput,
    pool: &tauri::State<'_, Pool<Sqlite>>,
) -> Result<UserResponseDB, String> {
    
    let user_db = user.new()?;

    info!(
        username = %user_db.username, 
        email = %user_db.email, 
        password = %user_db.password, 
        "struct: UserCreateDB, fn: new, proceeding to insert");

    let resp = user_query::user_post_query(&user_db, pool.inner())
        .await
        .map_err(|e| {
            error!(error = %e, "fn: user_post_query");
            e
        })?;

    Ok(resp)
}