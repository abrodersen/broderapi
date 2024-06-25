use std::env;

use eyre::{Context, Result};

#[derive(Clone)]
pub struct Config {
    pub matrix_user_id:  String,
    pub matrix_password: String,
    pub matrix_room_id:  String,
}

impl Config {
    pub fn create() -> Result<Self> {
        let user_id = env::var("MATRIX_USER_ID").wrap_err("failed to get user id")?;
        let password = env::var("MATRIX_PASSWORD").wrap_err("failed to get password")?;
        let room_id = env::var("MATRIX_ROOM_ID").wrap_err("failed to get room id")?;

        Ok(Config {
            matrix_user_id: user_id,
            matrix_password: password,
            matrix_room_id: room_id,
        })
    }
}
