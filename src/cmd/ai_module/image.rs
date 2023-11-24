use log::error;
use std::path::Path;
use std::{env, fs};

use crate::constant::COLOR;
use crate::function::error_management::error_base_url::error_no_base_url_edit;
use crate::function::error_management::error_creating_header::error_creating_header_edit;
use crate::function::error_management::error_getting_option::error_no_option;
use crate::function::error_management::error_instance_admin::error_instance_admin_models_edit;
use crate::function::error_management::error_not_nsfw::error_not_nsfw;
use crate::function::error_management::error_parsing_json::error_parsing_json_edit;
use crate::function::error_management::error_request::error_making_request_edit;
use crate::function::error_management::error_resolving_value::error_resolving_value_followup;
use crate::function::error_management::error_response::{
    error_getting_bytes_response_edit, error_getting_response_from_url_edit,
    error_writing_file_response_edit,
};
use crate::function::error_management::error_token::error_no_token_edit;
use crate::function::error_management::error_url::error_no_url_edit;
use crate::function::general::differed_response::differed_response;
use crate::function::general::get_nsfw_channel::get_nsfw;
use crate::function::general::in_progress::in_progress_embed;
use crate::structure::embed::ai::struct_lang_image::ImageLocalisedText;
use crate::structure::register::ai::struct_image_register::ImageRegister;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::channel::Message;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use uuid::Uuid;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let my_path = "./.env";
    let path = Path::new(my_path);
    let _ = dotenv::from_path(path);
    let nsfw_env = env::var("NSFW");
    match nsfw_env {
        Ok(nsfw) => {
            let is_ok = nsfw.to_lowercase() == "true";
            if !get_nsfw(command, ctx).await && is_ok {
                error_not_nsfw(ctx, command).await;
                return;
            }
        }
        Err(e) => {
            error!("{}", e);
            if !get_nsfw(command, ctx).await {
                error_not_nsfw(ctx, command).await;
                return;
            }
        }
    }
    let option = match options.get(0) {
        Some(data) => data,
        None => {
            error_no_option(ctx, command).await;
            return;
        }
    };
    let option = match option.resolved.as_ref() {
        Some(data) => data,
        None => {
            error_no_option(ctx, command).await;
            return;
        }
    };
    if let CommandDataOptionValue::String(description) = option {
        let uuid_name = Uuid::new_v4();
        let filename = format!("{}.png", uuid_name);
        let filename_str = filename.as_str();

        let localised_text = match ImageLocalisedText::get_image_localised(ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
        differed_response(ctx, command).await;

        let message = match in_progress_embed(ctx, command).await {
            Ok(Some(message_option)) => message_option,
            Ok(None) => {
                error_resolving_value_followup(ctx, command).await;
                return;
            }
            Err(error) => {
                error!("Error: {}", error);
                return;
            }
        };

        let prompt = description;
        let api_key = match env::var("AI_API_TOKEN") {
            Ok(x) => x,
            Err(why) => {
                error!("{}", why);
                error_no_token_edit(ctx, command, message).await;
                return;
            }
        };
        let api_base_url = match env::var("AI_API_BASE_URL") {
            Ok(x) => x,
            Err(why) => {
                error!("{}", why);
                error_no_base_url_edit(ctx, command, message).await;
                return;
            }
        };
        let data = match do_json(ctx, command, message.clone(), prompt).await {
            Ok(a) => a,
            Err(_) => return
        };
        let api_url = format!("{}images/generations", api_base_url);
        let client = reqwest::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            match HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                Ok(data) => data,
                Err(why) => {
                    error!("{}", why);
                    error_creating_header_edit(ctx, command, message).await;
                    return;
                }
            },
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res: Value = match client
            .post(api_url)
            .headers(headers)
            .json(&data)
            .send()
            .await
        {
            Ok(data) => match data.json().await {
                Ok(data) => data,
                Err(why) => {
                    println!("{}", why);
                    error_parsing_json_edit(ctx, message, command).await;
                    return;
                }
            },
            Err(why) => {
                error!("{}", why);
                error_making_request_edit(ctx, command, message).await;
                return;
            }
        };

        let mut url_string = "";
        if let Some(data) = res.get("data") {
            if let Some(object) = data.get(0) {
                if let Some(url) = object.get("url") {
                    url_string = match url.as_str() {
                        Some(url) => url,
                        None => {
                            error_no_url_edit(ctx, command, message).await;
                            return;
                        }
                    }
                }
            }
        }

        let real_message = message.clone();
        let response = match reqwest::get(url_string).await {
            Ok(data) => data,
            Err(why) => {
                error!("{}", why);
                error_getting_response_from_url_edit(ctx, command, message).await;
                return;
            }
        };
        let bytes = match response.bytes().await {
            Ok(data) => data,
            Err(why) => {
                error!("{}", why);
                error_getting_bytes_response_edit(ctx, command, message).await;
                return;
            }
        };
        match fs::write(&filename, &bytes) {
            Ok(_) => {}
            Err(why) => {
                error!("{}", why);
                error_writing_file_response_edit(ctx, command, message).await;
                return;
            }
        }


        send_embed(real_message, ctx, &filename, filename_str, localised_text).await;
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let images = match ImageRegister::get_image_register_localised() {
        Ok(images) => images,
        Err(e) => {
            error!("error when creating images command: {}", e);
            return command;
        }
    };
    command
        .name("image")
        .description("generate an image")
        .create_option(|option| {
            option
                .name("description")
                .description("Description of the image you want to generate.")
                .kind(CommandOptionType::String)
                .required(true);
            for image in images.values() {
                option
                    .name_localized(&image.code, &image.option1)
                    .description_localized(&image.code, &image.option1_desc);
            }
            option
        });
    for image in images.values() {
        command
            .name_localized(&image.code, &image.name)
            .description_localized(&image.code, &image.desc);
    }
    command
}

async fn send_embed(mut real_message: Message, ctx: &Context, filename: &String, filename_str: &str, localised_text: ImageLocalisedText) {
    let path = Path::new(filename_str);

    if let Err(why) = real_message
            .edit(&ctx.http, |m| {
                m.attachment(path).embed(|e| {
                    e.title(&localised_text.title)
                        .image(format!("attachment://{}", filename))
                        .timestamp(Timestamp::now())
                        .color(COLOR)
                })
            })
            .await
        {
            let _ = fs::remove_file(filename_str);
            error!("Cannot respond to slash command: {}", why);
        }
        let _ = fs::remove_file(filename_str);
}

async fn do_json(ctx: &Context, command: &ApplicationCommandInteraction, message: Message, prompt: &String) -> Result<Value, String> {
    let mut data = json!({
            "prompt": prompt,
            "n": 1,
            "size": "1024x1024",
            "response_format": "url"
        });
        if let Ok(image_generation_mode) = env::var("IMAGE_GENERATION_MODELS_ON") {
            let is_ok = image_generation_mode.to_lowercase() == "true";
            if is_ok {
                let model = match env::var("IMAGE_GENERATION_MODELS") {
                    Ok(data) => data,
                    Err(why) => {
                        error!("{}", why);
                        error_instance_admin_models_edit(ctx, command, message).await;
                        return Err(why.to_string())
                    }
                };
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": "1024x1024",
                    "model": model,
                    "response_format": "url"
                })
            }
        }
    Ok(data)
}