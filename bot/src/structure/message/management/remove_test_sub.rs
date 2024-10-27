use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct RemoveTestLocalised {
	pub success: String,
}

use anyhow::Result;

pub async fn load_localization_remove_test_sub(
	guild_id: String, db_config: DbConfig,
) -> Result<RemoveTestLocalised> {
	let path = "json/message/management/remove_test_sub.json";

	load_localization(guild_id, path, db_config).await
}
