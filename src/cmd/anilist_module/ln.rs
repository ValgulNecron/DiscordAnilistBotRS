use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};

use crate::cmd::anilist_module::command_media_ln::embed;
use crate::cmd::anilist_module::struct_autocomplete_media::MediaPageWrapper;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    return embed(options, ctx, command, "NOVEL").await;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("ln")
        .description("Info of a light novel")
        .create_option(|option| {
            option
                .name("ln_name")
                .description("Name of the light novel you want to check")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true)
        })
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_ln(search, 8, "MANGA", "NOVEL").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}
