use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp,
};

use crate::constant::COLOR;
use crate::database::dispatcher::data_dispatch::set_data_guild_langage;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{ErrorCommandSendingError, NoCommandOption, ErrorOptionError};
use crate::lang_struct::general::lang::load_localization_lang;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let lang = options
        .first()
        .ok_or(Error(ErrorOptionError(String::from("There is no option"))))?;
    let lang = lang.value.clone();

    let lang = match lang {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(Error(NoCommandOption(String::from(
                "The command contain no option.",
            ))));
        }
    };

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let _ = set_data_guild_langage(&guild_id, &lang).await;
    let lang_localised = load_localization_lang(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(lang_localised.desc.replace("$lang$", lang.as_str()))
        .title(&lang_localised.title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            Error(ErrorCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })
}
