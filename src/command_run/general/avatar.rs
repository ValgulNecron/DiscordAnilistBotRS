use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp, User,
};

use crate::constant::COLOR;
use crate::error_management::command_error::CommandError::Generic;
use crate::error_management::generic_error::GenericError::{OptionError, SendingCommand};
use crate::error_management::interaction_error::InteractionError;
use crate::lang_struct::general::avatar::load_localization_avatar;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    if let Some(option) = options.first() {
        let resolved = &option.value;
        if let CommandDataOptionValue::User(user, ..) = resolved {
            let user = user.to_user(&ctx.http).await.map_err(|e| {
                InteractionError::Command(Generic(OptionError(format!(
                    "Could not get the user. {}",
                    e
                ))))
            })?;
            return avatar_with_user(ctx, command_interaction, &user).await;
        }
    }
    avatar_without_user(ctx, command_interaction).await
}

async fn avatar_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let user = command_interaction.user.clone();
    avatar_with_user(ctx, command_interaction, &user).await
}

async fn avatar_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), InteractionError> {
    let avatar_url = user.face();
    send_embed(avatar_url, ctx, command_interaction, user.name.clone()).await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: String,
) -> Result<(), InteractionError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let avatar_localised = load_localization_avatar(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(avatar_url)
        .title(avatar_localised.title.replace("$user$", username.as_str()));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            InteractionError::Command(Generic(SendingCommand(format!(
                "Error while sending the command {}",
                e
            ))))
        })
}
