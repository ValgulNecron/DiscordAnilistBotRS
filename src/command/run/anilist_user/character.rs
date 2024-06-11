use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context};

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character::{
    send_embed, Character, CharacterQuerry, CharacterQuerryVariables,
};

/// This asynchronous function runs the command interaction for retrieving information about a character.
///
/// It first retrieves the name or ID of the character from the command interaction options.
///
/// If the value is an integer, it treats it as an ID and retrieves the character with that ID.
/// If the value is not an integer, it treats it as a name and retrieves the character with that name.
///
/// It sends an embed with the character information as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the name or ID of the character from the command interaction options
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    // If the value is an integer, treat it as an ID and retrieve the character with that ID
    // If the value is not an integer, treat it as a name and retrieve the character with that name
    let data: Character = if value.parse::<i32>().is_ok() {
        get_character_by_id(value.parse::<i32>().unwrap()).await?
    } else {
        let value_clone = value.clone();
        let var = CharacterQuerryVariables {
            id: None,
            search: Some(&*value),
        };
        get_character(var, value_clone).await?
    };

    // Send an embed with the character information as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}

pub async fn get_character_by_id(value: i32) -> Result<Character, AppError> {
    let var = CharacterQuerryVariables {
        id: Some(value),
        search: None,
    };
    get_character(var, value.to_string()).await
}

pub async fn get_character<'a>(
    var: CharacterQuerryVariables<'a>,
    value: String,
) -> Result<Character, AppError> {
    let operation = CharacterQuerry::build(var);
    let data: GraphQlResponse<CharacterQuerry> = match make_request_anilist(operation, false).await
    {
        Ok(data) => match data.json::<GraphQlResponse<CharacterQuerry>>().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(?e);
                return Err(AppError {
                    message: format!("Error retrieving character with value: {} \n {}", value, e),
                    error_type: ErrorType::WebRequest,
                    error_response_type: ErrorResponseType::Message,
                });
            }
        },
        Err(e) => {
            tracing::error!(?e);
            return Err(AppError {
                message: format!("Error retrieving character with value: {} \n {}", value, e),
                error_type: ErrorType::WebRequest,
                error_response_type: ErrorResponseType::Message,
            });
        }
    };
    Ok(data.data.unwrap().character.unwrap())
}
