use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

use crate::common::get_guild_lang::get_guild_langage;
use crate::error_management::file::file_error::FileError::{NotFound, Parsing, Reading};
use crate::error_management::lang::lang_error::LangError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterLocalised {
    pub desc: String,
    pub date_of_birth: String,
}

pub async fn load_localization_character(
    guild_id: String,
) -> Result<CharacterLocalised, LangError> {
    let mut file = File::open("json/message/anilist/character.json")
        .map_err(|e| NotFound(format!("File character.json not found. {}", e)))?;

    let mut json = String::new();
    file.read_to_string(&mut json)
        .map_err(|e| Reading(format!("File character.json can't be read. {}", e)))?;

    let json_data: HashMap<String, CharacterLocalised> = serde_json::from_str(&json)
        .map_err(|e| Parsing(format!("Failing to parse character.json. {}", e)))?;

    let lang_choice = get_guild_langage(guild_id).await;

    let localised_text = json_data
        .get(lang_choice.as_str())
        .ok_or(LangError::NotFound())?;

    Ok(localised_text.clone())
}
