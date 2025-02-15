use crate::command::command_trait::{Command, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::vndbapi::stats::get_stats;
use crate::structure::message::vn::stats::load_localization_stats;
use anyhow::Result;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};

pub struct VnStatsCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for VnStatsCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for VnStatsCommand {
	async fn run_slash(&self) -> Result<()> {
		send_embed(&self.ctx, &self.command_interaction).await
	}
}

async fn send_embed(ctx: &SerenityContext, command_interaction: &CommandInteraction) -> Result<()> {
	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let bot_data = ctx.data::<BotData>().clone();
	let vndb_cache = bot_data.vndb_cache.clone();
	let stats = get_stats(vndb_cache).await?;
	let config = bot_data.config.clone();
	let stats_localised = load_localization_stats(guild_id, config.db.clone()).await?;
	let fields = vec![
		(stats_localised.chars.clone(), stats.chars.to_string(), true),
		(
			stats_localised.producer.clone(),
			stats.producers.to_string(),
			true,
		),
		(
			stats_localised.release.clone(),
			stats.releases.to_string(),
			true,
		),
		(stats_localised.staff.clone(), stats.staff.to_string(), true),
		(stats_localised.staff.clone(), stats.staff.to_string(), true),
		(stats_localised.tags.clone(), stats.tags.to_string(), true),
		(
			stats_localised.traits.clone(),
			stats.traits.to_string(),
			true,
		),
		(stats_localised.vns.clone(), stats.vn.to_string(), true),
		(stats_localised.api.clone(), String::from("VNDB API"), true),
	];

	let builder_embed = get_default_embed(None)
		.title(stats_localised.title)
		.fields(fields);

	let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

	let builder = CreateInteractionResponse::Message(builder_message);

	command_interaction
		.create_response(&ctx.http, builder)
		.await?;

	Ok(())
}
