use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::CommandError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLocalised {
    pub manga: String,
    pub anime: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub weeks: String,
    pub days: String,
    pub hours: String,
    pub minutes: String,
}

pub async fn load_localization_user(guild_id: String) -> Result<UserLocalised, AppError> {
    let mut file = File::open("json/message/anilist/user.json").map_err(|e| {
        Error(LocalisationFileError(format!(
            "File user.json not found. {}",
            e
        )))
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        Error(LocalisationReadError(format!(
            "File user.json can't be read. {}",
            e
        )))
    })?;

    let json_data: HashMap<String, UserLocalised> = serde_json::from_str(&json).map_err(|e| {
        Error(LocalisationParsingError(format!(
            "Failing to parse user.json. {}",
            e
        )))
    })?;

    trace!("{}", guild_id);
    trace!("{}", guild_id != *"0");

    let lang_choice = get_guild_langage(guild_id).await;

    let user_localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(Error(NoLangageError(String::from("not found"))))?;

    Ok(user_localised_text.clone())
}
