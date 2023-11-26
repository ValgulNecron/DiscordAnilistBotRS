use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::Timestamp;

use crate::constant::COLOR;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::structure::embed::general::struct_lang_info::InfoLocalisedText;
use crate::structure::register::general::struct_info_register::RegisterLocalisedInfo;

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = InfoLocalisedText::get_info_localised(guild_id).await?;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.embed(
                    |m| {
                        m.title(&localised_text.title)
                            .description(&localised_text.description)
                            .footer(|f| f.text(&localised_text.footer))
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                    })
                    .components(|components| {
                        components.create_action_row(|row| {
                            row.create_button(|button| {
                                button.label(&localised_text.button_see_on_github)
                                    .url("https://github.com/ValgulNecron/DIscordAnilistBotRS")
                                    .style(ButtonStyle::Link)
                            })
                                .create_button(|button| {
                                    button.label(&localised_text.button_official_website)
                                        .url("https://kasuki.valgul.moe/")
                                        .style(ButtonStyle::Link)
                                })
                        })
                            .create_action_row(|button| {
                                button.create_button(|button| {
                                    button.label(&localised_text.button_official_discord)
                                        .url("https://discord.gg/dWGU6mkw7J")
                                        .style(ButtonStyle::Link)
                                })
                                    .create_button(|button| {
                                        button.label(&localised_text.button_add_the_bot)
                                            .url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot")
                                            .style(ButtonStyle::Link)
                                    })
                            })
                    })
                )
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let infos = RegisterLocalisedInfo::get_info_register_localised().unwrap();
    command
        .name("info")
        .description("Get information on the bot");
    for info in infos.values() {
        command
            .name_localized(&info.code, &info.name)
            .description_localized(&info.code, &info.desc);
    }
    command
}
