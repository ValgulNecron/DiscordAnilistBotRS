use std::u32;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;

#[derive(Deserialize)]
struct ApiResponse {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "Character")]
    character: Character,
}

#[derive(Deserialize)]
struct Character {
    id: u32,
    name: Name,
    #[serde(rename = "siteUrl")]
    site_url: String,
    description: String,
    gender: String,
    age: String,
    #[serde(rename = "dateOfBirth")]
    date_of_birth: DateOfBirth,
    image: Image,
    favourites: u32,
    #[serde(rename = "modNotes")]
    mod_notes: Option<String>,
}

#[derive(Deserialize)]
struct Name {
    full: String,
    native: String,
    #[serde(rename = "userPreferred")]
    user_preferred: String,
}

#[derive(Deserialize)]
struct DateOfBirth {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
}

#[derive(Deserialize)]
struct Image {
    large: String,
}

const QUERY: &str = "
query ($name: String) {
	Character(search: $name) {
    id
    name {
      full
      native
      userPreferred
    }
    siteUrl
    description
    gender
    age
    dateOfBirth {
      year
      month
      day
    }
    image {
      large
    }
    favourites
    modNotes
  }
}
";

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(name) = option {
        let client = Client::new();
        let json = json!({"query": QUERY, "variables": {"name": name}});
        let resp = client.post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        let data: ApiResponse = serde_json::from_str(&resp.unwrap()).unwrap();
        let color = Colour::FABLED_PINK;

        let name = format!("{}/{}", data.data.character.name.user_preferred, data.data.character.name.native);
        let desc = data.data.character.description;

        let image = data.data.character.image.large;
        let url = data.data.character.site_url;

        let age = data.data.character.age;
        let date_of_birth = format!("{}/{}/{}", data.data.character.date_of_birth.month.unwrap_or_else(|| 0),
                                    data.data.character.date_of_birth.day.unwrap_or_else(|| 0), data.data.character.date_of_birth.year.unwrap_or_else(|| 0));
        let gender = data.data.character.gender;
        let favourite = data.data.character.favourites;

        let full_description = format!("Age: {}. \n Gender: {}. \n Date of birth: {}. \n\
        Number of favourite: {}. \n Description: {}.", age, gender, date_of_birth, favourite
                                       , desc);

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.embed(
                        |m| {
                            m.title(name)
                                .url(url)
                                .timestamp(Timestamp::now())
                                .color(color)
                                .description(full_description)
                                .thumbnail(image)
                                .color(color)
                        })
                    )
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("character").description("Get information ").create_option(
        |option| {
            option
                .name("name")
                .description("The name of the character")
                .kind(CommandOptionType::String)
                .required(true)
        }
    )
}