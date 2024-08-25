use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::command::command_trait::Embed;
use crate::command::command_trait::{Command, EmbedType, SlashCommand};
use crate::config::{BotConfigDetails, Config};
use crate::get_url;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim_webhook;
use crate::structure::database::activity_data;
use crate::structure::database::activity_data::Column;
use crate::structure::database::prelude::ActivityData;
use crate::structure::message::admin::anilist::add_activity::{
    load_localization_add_activity
};
use crate::structure::run::anilist::minimal_anime::{
    Media, MediaTitle, MinimalAnimeId, MinimalAnimeIdVariables, MinimalAnimeSearch,
    MinimalAnimeSearchVariables,
};
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::Engine as _;
use chrono::Utc;
use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use moka::future::Cache;
use prost::bytes::Bytes;
use reqwest::get;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde_json::json;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, EditWebhook, GuildId,
};
use tokio::sync::RwLock;
use tracing::{error, trace};

pub struct AddActivityCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}
impl Command for AddActivityCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for AddActivityCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let anilist_cache = self.anilist_cache.clone();
        let command_interaction = self.command_interaction.clone();
        let ctx = self.ctx.clone();
        let config = self.config.clone();

        let map = get_option_map_string_subcommand_group(&command_interaction);
        let anime = map
            .get(&String::from("anime_name"))
            .cloned()
            .unwrap_or(String::new());
        let media = get_minimal_anime_media(anime.to_string(), anilist_cache).await?;

        let guild_id = match command_interaction.guild_id {
            Some(id) => id.to_string(),
            None => String::from("1"),
        };
        trace!(?guild_id);

        let add_activity_localised =
            load_localization_add_activity(guild_id.clone(), config.bot.config.clone()).await?;

        let anime_id = media.id;

        let builder_message = Defer(CreateInteractionResponseMessage::new());

        let exist =
            check_if_activity_exist(anime_id, guild_id.clone(), config.bot.config.clone()).await;

        command_interaction
            .create_response(&ctx.http, builder_message)
            .await?;
        let title = media
            .title
            .ok_or(error_dispatch::Error::Option("No title".to_string()))?;
        let anime_name = get_name(title);
        if exist {
            let url = format!("https://anilist.co/anime/{}", media.id);
            self.send_embed(
                Vec::new(),
                None,
                add_activity_localised.fail.clone(),
                add_activity_localised
                    .fail_desc
                    .replace("$anime$", anime_name.as_str()),
                None,
                Some(url),
                EmbedType::Followup,
                None,
            )
                .await?;
        } else {
            let channel_id = command_interaction.channel_id;

            let delay = map
                .get(&String::from("delay"))
                .unwrap_or(&String::from("0"))
                .parse()
                .unwrap_or(0);

            let trimmed_anime_name = if anime_name.len() >= 50 {
                trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
            } else {
                anime_name.clone()
            };

            let bytes = get(media.cover_image.ok_or(
                error_dispatch::Error::Option("No cover image".to_string()),
            )?.extra_large.
                unwrap_or(
                    "https://imgs.search.brave.com/ CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
                        .to_string()
                )
            ).await?.bytes().await?;

            let buf = resize_image(&bytes).await?;
            let base64 = STANDARD.encode(buf.into_inner());
            let image = format!("data:image/jpeg;base64,{}", base64);

            let next_airing = match media.next_airing_episode.clone() {
                Some(na) => na,
                None => {
                    return Err(Box::new(error_dispatch::Error::Option(format!(
                        "No next episode found for {} on anilist",
                        anime_name
                    ))));
                }
            };
            let webhook = get_webhook(
                &ctx,
                channel_id,
                image.clone(),
                base64.clone(),
                trimmed_anime_name.clone(),
            )
                .await?;
            let connection = sea_orm::Database::connect(get_url(config.bot.config.clone())).await?;
            let timestamp = next_airing.airing_at as i64;

            let chrono = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_default()
                .naive_utc();
            ActivityData::insert(activity_data::ActiveModel {
                anime_id: Set(media.id),
                timestamp: Set(chrono),
                server_id: Set(guild_id),
                webhook: Set(webhook),
                episode: Set(next_airing.episode),
                name: Set(trimmed_anime_name),
                delay: Set(delay),
                image: Set(image),
                ..Default::default()
            })
                .exec(&connection)
                .await?;
            let url = format!("https://anilist.co/anime/{}", media.id);

            self.send_embed(
                Vec::new(),
                None,
                add_activity_localised.success.clone(),
                add_activity_localised
                    .success_desc
                    .replace("$anime$", anime_name.as_str()),
                None,
                Some(url),
                EmbedType::Followup,
                None,
            )
                .await?;
        }

        Ok(())
    }
}

async fn resize_image(image_bytes: &Bytes) -> Result<Cursor<Vec<u8>>, Box<dyn Error>> {
    let image = image::load(Cursor::new(image_bytes), guess_format(image_bytes)?)?;
    let (image_width, image_height) = image.dimensions();
    let square_size = image_width.min(image_height);
    let crop_x = (image_width - square_size) / 2;
    let crop_y = (image_height - square_size) / 2;

    let resized_image = image
        .crop_imm(crop_x, crop_y, square_size, square_size)
        .resize_exact(128, 128, FilterType::Nearest);

    let mut buffer = Cursor::new(Vec::new());
    resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;

    Ok(buffer)
}

async fn check_if_activity_exist(
    anime_id: i32,
    server_id: String,
    db_config: BotConfigDetails,
) -> bool {
    let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
        Ok(conn) => conn,
        Err(_) => {
            return false;
        }
    };
    let row = match ActivityData::find()
        .filter(Column::ServerId.eq(server_id))
        .filter(Column::AnimeId.eq(anime_id))
        .one(&connection)
        .await
    {
        Ok(row) => row,
        Err(_) => {
            return false;
        }
    };
    if row.is_none() {
        return false;
    };
    true
}

pub fn get_name(title: MediaTitle) -> String {
    let en = title.english.clone();
    let rj = title.romaji.clone();

    match (rj, en) {
        (Some(rj), Some(en)) => format!("{} / {}", en, rj),
        (Some(rj), None) => rj,
        (None, Some(en)) => en,
        (None, None) => String::new(),
    }
}

async fn get_webhook(
    ctx: &Context,
    channel_id: ChannelId,
    image: String,
    base64: String,
    anime_name: String,
) -> Result<String, Box<dyn Error>> {
    let map = json!({
        "avatar": image,
        "name": anime_name
    });

    let bot_id = match ctx.http.get_current_application_info().await {
        Ok(bot_info) => bot_info.id.to_string(),
        Err(e) => {
            error!("{}", e);
            String::new()
        }
    };

    trace!(bot_id);
    let mut webhook_return = String::new();

    let webhooks = match ctx.http.get_channel_webhooks(channel_id).await {
        Ok(vec) => vec,
        Err(_) => {
            let webhook = ctx.http.create_webhook(channel_id, &map, None).await?;
            webhook_return = webhook.url()?;

            return Ok(webhook_return);
        }
    };
    if webhooks.is_empty() {
        let webhook = ctx.http.create_webhook(channel_id, &map, None).await?;
        webhook_return = webhook.url()?;

        return Ok(webhook_return);
    }
    for webhook in webhooks {
        trace!("{:#?}", webhook);
        let webhook_user_id = webhook
            .user
            .clone()
            .ok_or(Box::new(error_dispatch::Error::Webhook(
                "webhook user id not found".to_string(),
            )))?
            .id
            .to_string();
        trace!(webhook_user_id);
        if webhook_user_id == bot_id {
            trace!("Getting webhook");
            webhook_return = webhook.url()?;
        } else {
            trace!(webhook_return);
            let is_ok = webhook_return == String::new();
            trace!(is_ok);
            if is_ok {
                trace!("Creating webhook");
                let webhook = ctx.http.create_webhook(channel_id, &map, None).await?;
                webhook_return = webhook.url()?;
            }
        }
    }
    trace!("Done");
    trace!(webhook_return);
    let cursor = Cursor::new(base64);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    // Read the decoded bytes into a Vec
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes)?;
    let mut webhook = ctx
        .http
        .get_webhook_from_url(webhook_return.as_str())
        .await?;
    let attachement = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new().name(anime_name).avatar(&attachement);
    webhook.edit(&ctx.http, edit_webhook).await?;

    Ok(webhook_return)
}

pub(crate) async fn get_minimal_anime_by_id(
    id: i32,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let query = MinimalAnimeIdVariables { id: Some(id) };
    let operation = MinimalAnimeId::build(query);
    let data: GraphQlResponse<MinimalAnimeId> =
        make_request_anilist(operation, false, anilist_cache).await?;
    let data = data
        .data
        .ok_or(error_dispatch::Error::Option("No media found".to_string()))?
        .media
        .ok_or(error_dispatch::Error::Option("No media found".to_string()))?;
    Ok(data)
}

async fn get_minimal_anime_by_search(
    value: &str,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let query = MinimalAnimeSearchVariables {
        search: Some(value),
    };
    let operation = MinimalAnimeSearch::build(query);
    let data: GraphQlResponse<MinimalAnimeSearch> =
        make_request_anilist(operation, false, anilist_cache).await?;
    Ok(match data.data {
        Some(data) => match data.media {
            Some(media) => media,
            None => {
                return Err(Box::new(error_dispatch::Error::Option(
                    "No media found".to_string(),
                )))
            }
        },
        None => {
            return Err(Box::new(error_dispatch::Error::Option(
                "No data found".to_string(),
            )))
        }
    })
}

pub async fn get_minimal_anime_media(
    anime: String,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let media = if anime.parse::<i32>().is_ok() {
        get_minimal_anime_by_id(anime.parse::<i32>().unwrap_or_default(), anilist_cache).await?
    } else {
        get_minimal_anime_by_search(anime.as_str(), anilist_cache).await?
    };
    Ok(media)
}
