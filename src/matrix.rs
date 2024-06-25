use crate::cfg::Config;

use eyre::{Context, Result};
use matrix_sdk::{ruma::UserId, Client};

pub async fn init_matrix_client(settings: &Config) -> Result<Client> {
    let user_id = UserId::parse(&settings.matrix_user_id)
        .wrap_err("failed to parse matrix user id")?;
    let client = Client::builder()
        .server_name(user_id.server_name())
        .build()
        .await
        .wrap_err("failed to initialize matrix client")?;

    // println!("running preliminary sync");
    // client.sync_once(SyncSettings::default())
    //     .await
    //     .wrap_err("failed to perform initial client sync")?;

    Ok(client)
}