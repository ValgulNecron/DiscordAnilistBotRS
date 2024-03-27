
use crate::command_register::command_struct::subcommand::SubCommand;
use crate::command_register::registration_function::common::{get_subcommand_option};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use serenity::all::{CreateCommand, Http, Permissions};
use std::fs;
use std::io::BufReader;
use std::sync::Arc;
use tracing::{error, trace};

pub async fn creates_subcommands(http: &Arc<Http>) {
    let commands = match get_subcommands("./json/subcommand") {
        Err(e) => {
            error!("{:?}", e);
            return;
        }
        Ok(c) => c,
    };

    for command in commands {
        create_command(&command, http).await;
    }
}

fn get_subcommands(path: &str) -> Result<Vec<SubCommand>, AppError> {
    let mut subcommands = Vec::new();
    let paths = fs::read_dir(path).map_err(|e| AppError {
        message: format!("Failed to read directory: {:?} with error {}", path, e),
        error_type: ErrorType::File,
        error_response_type: ErrorResponseType::None,
    })?;
    for entry in paths {
        let entry = entry.map_err(|e| AppError {
            message: format!("Failed to read path with error {}", e),
            error_type: ErrorType::File,
            error_response_type: ErrorResponseType::None,
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let file = fs::File::open(path.as_path()).map_err(|e| AppError {
                message: format!("Failed to open file: {:?} with error {}", path.as_path(), e),
                error_type: ErrorType::File,
                error_response_type: ErrorResponseType::None,
            })?;
            let reader = BufReader::new(file);
            let command: SubCommand = serde_json::from_reader(reader).map_err(|e| AppError {
                message: format!(
                    "Failed to parse file: {:?} with error {}",
                    path.as_path(),
                    e
                ),
                error_type: ErrorType::File,
                error_response_type: ErrorResponseType::None,
            })?;
            subcommands.push(command);
        }
    }
    if subcommands.is_empty() {
        trace!("No subcommands found in the directory: {:?}", path);
    }
        Ok(subcommands)
}

async fn create_command(command: &SubCommand, http: &Arc<Http>) {
    let mut command_build = CreateCommand::new(&command.name)
        .nsfw(command.nsfw)
        .dm_permission(command.dm_command)
        .description(&command.desc);

    let mut permission = Permissions::empty();
    if let Some(permissions) = &command.permissions {
        let mut perm_bit: u64 = 0;
        for perm in permissions {
            let permission: Permissions = perm.permission.into();
            perm_bit |= permission.bits()
        }
        permission = Permissions::from_bits(perm_bit).unwrap()
    }
    command_build = command_build.default_member_permissions(permission);
    command_build = match &command.command {
        Some(command) => {
            let options = get_subcommand_option(command);
            command_build.set_options(options)
        }
        None => command_build,
    };

    let e = http.create_global_command(&command_build).await;
    match e {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create command: {:?}", e);
        }
    }
}
