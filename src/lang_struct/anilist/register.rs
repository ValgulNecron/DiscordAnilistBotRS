use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalised {
    pub desc: String,
}

pub async fn load_localization_register(guild_id: String) -> Result<RegisterLocalised, AppError> {
    let mut file = File::open("json/message/anilist/register.json").map_err(|e| {
        AppError::new(
            format!("File register.json not found. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| {
        AppError::new(
            format!("File register.json can't be read. {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;

    let json_data: HashMap<String, RegisterLocalised> =
        serde_json::from_str(&json).map_err(|e| {
            AppError::new(
                format!("Failing to parse register.json. {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?;

    let lang_choice = get_guild_langage(guild_id).await;

    let localised_text = json_data.get(lang_choice.as_str()).ok_or(AppError::new(
        "Language not found.".to_string(),
        ErrorType::Language,
        ErrorResponseType::Unknown,
    ))?;

    Ok(localised_text.clone())
}
