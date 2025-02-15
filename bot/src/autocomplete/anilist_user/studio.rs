use crate::constant::DEFAULT_STRING;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::studio::{
	StudioAutocomplete, StudioAutocompleteVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
	AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
	CreateInteractionResponse,
};
use small_fixed_array::FixedString;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
	let map = get_option_map_string(&autocomplete_interaction);
	let bot_data = ctx.data::<BotData>().clone();

	let studio_search = map
		.get(&FixedString::from_str_trunc("studio"))
		.unwrap_or(DEFAULT_STRING);

	let var = StudioAutocompleteVariables {
		search: Some(studio_search),
	};

	let operation = StudioAutocomplete::build(var);

	let data: GraphQlResponse<StudioAutocomplete> =
		match make_request_anilist(operation, false, bot_data.anilist_cache.clone()).await {
			Ok(data) => data,
			Err(e) => {
				tracing::error!(?e);

				return;
			},
		};

	let studios = match data.data {
		Some(data) => match data.page {
			Some(page) => match page.studios {
				Some(studios) => studios,
				None => return,
			},
			None => return,
		},
		None => {
			tracing::debug!(?data.errors);

			return;
		},
	};

	let mut choices = Vec::new();

	for studio in studios {
		let data = studio.unwrap();

		let user = data.name;

		choices.push(AutocompleteChoice::new(user, data.id.to_string()))
	}

	let data = CreateAutocompleteResponse::new().set_choices(choices);

	let builder = CreateInteractionResponse::Autocomplete(data);

	let _ = autocomplete_interaction
		.create_response(&ctx.http, builder)
		.await;
}
