use crate::constant::COLOR;
use crate::database::sqlite::data::get_server_image_sqlite;

use crate::lang_struct::general::generate_image_pfp_server::load_localization_pfp_server_image;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;

use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};

use crate::error_management::command_error::CommandError::Generic;
use crate::error_management::generic_error::GenericError::{OptionError, SendingCommand};
use crate::error_management::interaction_error::InteractionError;
use tracing::trace;
use uuid::Uuid;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let pfp_server_image_localised_text =
        load_localization_pfp_server_image(guild_id.clone()).await?;
    let image = get_server_image_sqlite(&guild_id, &String::from("local"))
        .await?
        .1
        .unwrap_or_default();
    let input = image.trim_start_matches("data:image/png;base64,");
    let image_data: Vec<u8> = BASE64.decode(input).map_err(|e| {
        InteractionError::Command(Generic(OptionError(format!(
            "Error when decoding the image or there is no image {}",
            e
        ))))
    })?;
    let uuid = Uuid::new_v4();
    let image_path = format!("{}.png", uuid);

    let attachment = CreateAttachment::bytes(image_data, &image_path);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(pfp_server_image_localised_text.title);

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            InteractionError::Command(Generic(SendingCommand(format!(
                "Error while sending the command {}",
                e
            ))))
        })?;
    trace!("Done");

    Ok(())
}
