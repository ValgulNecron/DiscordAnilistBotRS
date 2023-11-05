use crate::constant::COLOR;
use crate::function::error_management::common::custom_error;
use crate::function::error_management::error_not_nsfw::error_not_nsfw;
use crate::function::general::get_nsfw_channel::get_nsfw;
use crate::structure::anilist::media::struct_media::MediaWrapper;
use crate::structure::embed::anilist::struct_lang_media::MediaLocalisedText;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;

pub async fn embed(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    search_type: &str,
) {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");
    if let CommandDataOptionValue::String(value) = option {
        let localised_text = match MediaLocalisedText::get_media_localised(ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
        let data: MediaWrapper;
        if value.parse::<i32>().is_ok() {
            if search_type == "NOVEL" {
                data =
                    match MediaWrapper::new_ln_by_id(value.parse().unwrap(), localised_text.clone())
                        .await
                    {
                        Ok(character_wrapper) => character_wrapper,
                        Err(error) => {
                            custom_error(ctx, command, &error).await;
                            return;
                        }
                    }
            } else {
                data = match MediaWrapper::new_manga_by_id(
                    value.parse().unwrap(),
                    localised_text.clone(),
                )
                .await
                {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => {
                        custom_error(ctx, command, &error).await;
                        return;
                    }
                }
            }
        } else if search_type == "NOVEL" {
            data =
                match MediaWrapper::new_ln_by_search(value.parse().unwrap(), localised_text.clone())
                    .await
                {
                    Ok(character_wrapper) => character_wrapper,
                    Err(error) => {
                        custom_error(ctx, command, &error).await;
                        return;
                    }
                }
        } else {
            data = match MediaWrapper::new_manga_by_search(
                value.parse().unwrap(),
                localised_text.clone(),
            )
            .await
            {
                Ok(character_wrapper) => character_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        }

        if data.get_nsfw() && !get_nsfw(command, ctx).await {
            error_not_nsfw(ctx, command).await;
            return;
        }

        let banner_image = format!("https://img.anili.st/media/{}", data.data.media.id);
        let desc = data.get_desc();
        let thumbnail = data.get_thumbnail();
        let site_url = data.get_url();
        let name = data.get_name();

        let info = data.get_media_info(localised_text.clone());
        let genre = data.get_genres();
        let tag = data.get_tags();
        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.embed(|m| {
                            m.title(name)
                                .url(site_url)
                                .timestamp(Timestamp::now())
                                .color(COLOR)
                                .description(desc)
                                .thumbnail(thumbnail)
                                .image(banner_image)
                                .field("Info", info, false)
                                .fields(vec![("Genre", genre, true), ("Tag", tag, true)])
                        })
                    })
            })
            .await
        {
            println!("Error creating slash command: {}", why);
        }
    }
}
