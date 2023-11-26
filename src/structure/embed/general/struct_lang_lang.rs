use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    LocalisationFileError, LocalisationParsingError, LocalisationReadError, NoLangageError,
};
use crate::function::general::get_guild_langage::get_guild_langage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangLocalisedText {
    pub title: String,
    pub description: String,
    pub error_perm: String,
}

impl LangLocalisedText {
    pub async fn get_ping_localised(guild_id: &String) -> Result<LangLocalisedText, AppError> {
        let mut file = File::open("./lang_file/embed/general/lang.json")
            .map_err(|_| LocalisationFileError(String::from("File lang.json not found.")))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| LocalisationReadError(String::from("File lang.json can't be read.")))?;

        let json_data: HashMap<String, LangLocalisedText> = serde_json::from_str(&json)
            .map_err(|_| LocalisationParsingError(String::from("Failing to parse lang.json.")))?;

        let lang_choice = get_guild_langage(guild_id).await;

        let avatar_localised_text = json_data
            .get(lang_choice.as_str())
            .ok_or(NoLangageError(String::from("not found")))?;

        Ok(avatar_localised_text.clone())
    }
}
