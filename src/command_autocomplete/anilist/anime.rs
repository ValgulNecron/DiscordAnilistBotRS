use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut search = String::new();
    for option in &command.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
    let anime = MediaPageWrapper::new_autocomplete_anime(&search.to_string()).await;
    send_auto_complete(ctx, command, anime).await;
}
