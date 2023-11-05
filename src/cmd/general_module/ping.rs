use crate::constant::COLOR;
use crate::structure::embed::general::struct_lang_ping::PingLocalisedText;
use crate::structure::register::general::struct_ping_register::RegisterLocalisedPing;
use serenity::builder::CreateApplicationCommand;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::Timestamp;

use crate::structure::struct_shard_manager::ShardManagerContainer;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let data_read = ctx.data.read().await;
    let shard_manager = match data_read.get::<ShardManagerContainer>() {
        Some(data) => data,
        None => return,
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(data) => data,
        None => return,
    };

    let latency = match runner.latency {
        Some(duration) => format!("{:.2}ms", duration.as_millis()),
        None => "?ms".to_string(),
    };

    let localised_text = match PingLocalisedText::get_ping_localised(ctx, command).await {
        Ok(data) => data,
        Err(_) => return,
    };
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(&localised_text.title)
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .description(format!(
                                "{}{}{}{}{}",
                                &localised_text.description_part_1,
                                &localised_text.description_part_2,
                                ctx.shard_id,
                                &localised_text.description_part_3,
                                latency
                            ))
                    })
                })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let pings = RegisterLocalisedPing::get_ping_register_localised().unwrap();
    let command = command.name("ping").description("A ping command");
    for ping in pings.values() {
        command
            .name_localized(&ping.code, &ping.name)
            .description_localized(&ping.code, &ping.desc);
    }
    command
}
