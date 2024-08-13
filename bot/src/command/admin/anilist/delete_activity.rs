use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::{BotConfigDetails, Config};
use crate::database::dispatcher::data_dispatch::remove_data_activity_status;
use crate::helper::create_default_embed::get_anilist_anime_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::anilist::delete_activity::load_localization_delete_activity;

pub struct DeleteActivityCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for DeleteActivityCommand {
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }

    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
}

impl SlashCommand for DeleteActivityCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let anilist_cache = self.anilist_cache.clone();
        let command_interaction = self.command_interaction.clone();
        send_embed(
            &self.ctx,
            &command_interaction,
            self.config.clone(),
            anilist_cache,
        )
        .await
    }
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let map = get_option_map_string_subcommand_group(command_interaction);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let delete_activity_localised_text = load_localization_delete_activity(
        guild_id.clone(),
        db_type.clone(),
        config.bot.config.clone(),
    )
    .await?;
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    trace!(anime);
    let media = get_minimal_anime_media(anilist_cache, command_interaction).await?;

    let anime_id = media.id;
    remove_activity(
        guild_id.as_str(),
        &anime_id,
        db_type,
        config.bot.config.clone(),
    )
    .await?;

    let title = media.title.unwrap();
    let anime_name = get_name(title);
    let builder_embed = get_anilist_anime_embed(None, media.id)
        .title(&delete_activity_localised_text.success)
        .description(
            delete_activity_localised_text
                .success_desc
                .replace("$anime$", anime_name.as_str()),
        );

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
    Ok(())
}

/// This asynchronous function removes an activity for a given anime and server from the database.
///
/// # Arguments
///
/// * `guild_id` - The ID of the server from which to remove the activity.
/// * `anime_id` - The ID of the anime for which to remove the activity.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
async fn remove_activity(
    guild_id: &str,
    anime_id: &i32,
    db_type: String,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    remove_data_activity_status(
        guild_id.to_owned(),
        anime_id.to_string(),
        db_type,
        db_config,
    )
    .await?;
    Ok(())
}
