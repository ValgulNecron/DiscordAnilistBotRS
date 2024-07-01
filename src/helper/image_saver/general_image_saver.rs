// Importing necessary libraries and modules
use std::env;

use crate::helper::error_management::error_enum::AppError;
use crate::helper::image_saver::catbox_image_saver::upload_image_catbox;
use crate::helper::image_saver::local_image_saver::local_image_save;

/// `image_saver` is an asynchronous function that saves an image either locally or remotely.
/// It takes a `guild_id`, `filename`, and `image_data` as parameters.
/// `guild_id` and `filename` are both Strings, and `image_data` is a Vec<u8>.
/// It returns a Result which is either an empty tuple or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string that represents the guild id.
/// * `filename` - A string that represents the filename of the image.
/// * `image_data` - A Vec<u8> that represents the image data.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while saving the image.
pub async fn image_saver(
    guild_id: String,
    filename: String,
    image_data: Vec<u8>,
) -> Result<(), AppError> {
    // Get the type of saver from the environment variables
    let saver_type = env::var("SAVE_IMAGE").unwrap_or("local".to_string());
    // If the saver type is local, save the image locally
    if saver_type == *"local" {
        local_image_save(guild_id, filename, image_data).await
        // If the saver type is remote, save the image remotely
    } else if saver_type == *"remote" {
        remote_saver(filename, image_data).await
    } else {
        Ok(())
    }
}

/// `remote_saver` is an asynchronous function that saves an image remotely.
/// It takes a `filename` and `image_data` as parameters.
/// `filename` is a String, and `image_data` is a Vec<u8>.
/// It returns a Result which is either an empty tuple or an `AppError`.
///
/// # Arguments
///
/// * `filename` - A string that represents the filename of the image.
/// * `image_data` - A Vec<u8> that represents the image data.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while saving the image remotely.
pub async fn remote_saver(filename: String, image_data: Vec<u8>) -> Result<(), AppError> {
    // Get the server for saving from the environment variables
    let saver_server = env::var("SAVE_SERVER").unwrap_or("catbox".to_string());
    // If the server is catbox, upload the image to catbox
    if saver_server == *"catbox" {
        upload_image_catbox(filename, image_data).await
        // If the server is imgur, upload the image to imgur
    } else {
        upload_image_catbox(filename, image_data).await
    }
}
