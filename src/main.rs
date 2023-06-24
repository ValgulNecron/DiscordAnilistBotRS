extern crate core;

use std::env;

use serenity::async_trait;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Activity;
use serenity::model::gateway::ActivityType;
use serenity::model::gateway::Ready;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;

use crate::cmd::ai_module::*;
use crate::cmd::anilist_module::*;
use crate::cmd::general_module::*;

mod cmd;

struct Handler;

const ACTIVITY_NAME: &str = "Do /help to get the list of command";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Add activity to the bot as the type in activity_type and with ACTIVITY_NAME as name
        let activity_type: Activity = Activity::playing(ACTIVITY_NAME);
        ctx.set_activity(activity_type).await;

        println!("{} is connected!", ready.user.name);

        // Create all the commande found in cmd. (if a command is added it will need to be added here).
        let guild_command = Command::set_global_application_commands(&ctx.http, |commands|
            {
                commands
                    .create_application_command(|command| ping::register(command))
                    .create_application_command(|command| info::register(command))


                    .create_application_command(|command| manga::register(command))
                    .create_application_command(|command| ln::register(command))
                    .create_application_command(|command| anime::register(command))
                    .create_application_command(|command| user::register(command))
                    .create_application_command(|command| level::register(command))
                    .create_application_command(|command| compare::register(command))
                    .create_application_command(|command| random::register(command))
                    .create_application_command(|command| staff::register(command))


                    .create_application_command(|command| image::register(command))
            }).await;

        println!("I created the following global slash command: {:#?}", guild_command);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {

                // Check which command was called and dispatch it to it run function.
                "ping" => {
                    ping::run(&command.data.options)
                }
                "info" => {
                    info::run(&command.data.options, &ctx, &command)
                        .await
                }




                "anime" => {
                    anime::run(&command.data.options, &ctx, &command).await
                }
                "compare" => {
                    compare::run(&command.data.options, &ctx, &command)
                        .await
                }
                "level" => {
                    level::run(&command.data.options, &ctx, &command)
                        .await
                }
                "ln" => {
                    ln::run(&command.data.options, &ctx, &command).await
                }
                "manga" => {
                    manga::run(&command.data.options, &ctx, &command).await
                }
                "random" => {
                    random::run(&command.data.options, &ctx, &command)
                        .await
                }
                "staff" => {
                    staff::run(&command.data.options, &ctx, &command)
                        .await
                }
                "user" => {
                    user::run(&command.data.options, &ctx, &command)
                        .await
                }





                "image" => {
                    image::run(&command.data.options, &ctx, &command)
                        .await
                }

                _ => "not implemented :(".to_string(),
            };

            // check if the command was successfully done.
            if content == "good".to_string() {
                return;
            }
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let my_path = ".\\src\\.env";
    println!("{}", my_path.to_string());
    let path = std::path::Path::new(my_path);
    dotenv::from_path(path);
    let token = env::var("DISCORD_TOKEN").expect("discord token");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}