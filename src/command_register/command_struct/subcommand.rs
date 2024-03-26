use crate::command_register::command_struct::common::{
    Arg, DefaultPermission, Localised, RemoteCommandOptionType, RemotePermissionType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubCommand {
    pub name: String,
    pub desc: String,
    pub dm_command: bool,
    pub nsfw: bool,
    pub permissions: Option<Vec<DefaultPermission>>,
    pub command: Option<Vec<Command>>,
    pub localised: Option<Vec<Localised>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub desc: String,
    pub args: Option<Vec<Arg>>,
    pub localised: Option<Vec<Localised>>,
}
