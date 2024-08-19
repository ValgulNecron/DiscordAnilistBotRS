use std::error::Error;
use std::io::Cursor;
use std::ptr::replace;
use std::sync::Arc;
use std::time::Duration;

use crate::config::BotConfigDetails;
use crate::get_url;
use crate::structure::database::prelude::{UserColor, UserData};
use crate::structure::database::user_color::{ActiveModel, Column, Model};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use image::codecs::png::PngEncoder;
use image::ImageReader;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use sea_orm::sea_query::OnConflict;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{Context, GuildId, Member, UserId};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, trace};

/// Calculates the color for each user in a list of members.
///
/// This function iterates over a list of members, skipping any members that are in the `USER_BLACKLIST_SERVER_IMAGE`.
/// For each member, it retrieves the user's profile picture URL and the user's ID.
/// It then retrieves the user's color from the database.
/// If the profile picture URL has changed since the last time the color was calculated, it recalculates the color and updates the database.
/// After processing each member, it sleeps for 100 milliseconds to avoid rate limiting.
///
/// # Arguments
///
/// * `members` - A vector of `Member` objects to calculate colors for.
///
/// # Returns
///
/// * `Result<(), AppError>` - On success, the function returns `Ok(())`.
///   If the function fails to calculate the color for any member, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if there's a problem retrieving the user's color from the database,
/// calculating the user's color, or updating the user's color in the database.
pub async fn calculate_users_color(
    members: Vec<Member>,
    user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    let guard = user_blacklist_server_image.read().await;
    for member in members {
        trace!("Calculating user color for {}", member.user.id);
        if guard.contains(&member.user.id.to_string()) {
            debug!(
                "Skipping user {} due to USER_BLACKLIST_SERVER_IMAGE",
                member.user.id
            );
            continue;
        }
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://cdn.discordapp.com/avatars/260706120086192129/ec231a35c9a33dd29ea4819d29d06056.webp?size=64"))
            .replace("?size=1024", "?size=64");

        let id = member.user.id.to_string();
        let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;
        let user_color = UserColor::find()
            .filter(Column::UserId.eq(id.clone()))
            .one(&connection)
            .await?
            .unwrap_or(Model {
                user_id: id.clone(),
                profile_picture_url: String::from(""),
                color: String::from(""),
                images: String::from(""),
                calculated_at: Default::default(),
            });
        let pfp_url_old = user_color.profile_picture_url.clone();
        if pfp_url != pfp_url_old {
            let (average_color, image): (String, String) =
                calculate_user_color(member.clone()).await?;
            UserColor::insert(ActiveModel {
                user_id: Set(id.clone()),
                profile_picture_url: Set(pfp_url.clone()),
                color: Set(average_color.clone()),
                images: Set(image.clone()),
                ..Default::default()
            })
            .on_conflict(
                OnConflict::column(Column::UserId)
                    .update_column(Column::Color)
                    .update_column(Column::ProfilePictureUrl)
                    .update_column(Column::Images)
                    .to_owned(),
            )
            .exec(&connection)
            .await?;
            match UserData::insert(crate::structure::database::user_data::ActiveModel {
                user_id: Set(id.clone()),
                username: Set(member.user.name),
                added_at: Set(Utc::now().naive_utc()),
                ..Default::default()
            })
            .on_conflict(
                OnConflict::column(crate::structure::database::user_data::Column::UserId)
                    .update_column(crate::structure::database::user_data::Column::Username)
                    .to_owned(),
            )
            .exec(&connection)
            .await
            {
                Ok(_) => {}
                Err(e) => error!("Failed to insert user data. {}", e),
            };
        }
        trace!("Done calculating user color for {}", member.user.id);
        sleep(Duration::from_nanos(1)).await
    }
    Ok(())
}

/// Returns the average color for each user in a list of members.
///
/// This function iterates over a list of members. For each member, it retrieves the user's profile picture URL and the user's ID.
/// It then retrieves the user's color from the database. If the profile picture URL has changed since the last time the color was calculated,
/// it recalculates the color and updates the database. The function then pushes the average color, profile picture URL, and image into a vector.
///
/// # Arguments
///
/// * `members` - A vector of `Member` objects to calculate colors for.
///
/// # Returns
///
/// * `Result<Vec<(String, String, String)>, AppError>` - On success, the function returns `Ok(Vec<(String, String, String)>)`, where each tuple in the vector represents a user's average color, profile picture URL, and image.
///   If the function fails to calculate the color for any member, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if there's a problem retrieving the user's color from the database,
/// calculating the user's color, or updating the user's color in the database.
pub async fn return_average_user_color(
    members: Vec<Member>,
    db_config: BotConfigDetails,
) -> Result<Vec<(String, String, String)>, Box<dyn Error>> {
    let mut average_colors = Vec::new();
    for member in members {
        let pfp_url = member.user.avatar_url().unwrap_or(String::from("https://cdn.discordapp.com/avatars/260706120086192129/ec231a35c9a33dd29ea4819d29d06056.webp?size=64"))
            .replace("?size=4096", "?size=64")
            .replace("?size=2048", "?size=64")
            .replace("?size=1024", "?size=64")
            .replace("?size=512", "?size=64")
            .replace("?size=256", "?size=64")
            .replace("?size=128", "?size=64");
        let id = member.user.id.to_string();

        let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;
        let user_color = UserColor::find()
            .filter(Column::UserId.eq(id.clone()))
            .one(&connection)
            .await?;

        match user_color {
            Some(user_color) => {
                let color = user_color.color.clone();
                let pfp_url_old = user_color.profile_picture_url.clone();
                let image_old = user_color.images;
                if pfp_url != pfp_url_old {
                    let (average_color, image): (String, String) =
                        calculate_user_color(member.clone()).await?;
                    average_colors.push((average_color.clone(), pfp_url.clone(), image.clone()));
                    UserColor::insert(ActiveModel {
                        user_id: Set(id.clone()),
                        profile_picture_url: Set(pfp_url.clone()),
                        color: Set(average_color.clone()),
                        images: Set(image.clone()),
                        ..Default::default()
                    })
                    .on_conflict(
                        // update url, color, images and calculated_at
                        OnConflict::column(Column::UserId)
                            .update_column(Column::Color)
                            .update_column(Column::ProfilePictureUrl)
                            .update_column(Column::Images)
                            .update_column(Column::CalculatedAt)
                            .to_owned(),
                    )
                    .exec(&connection)
                    .await?;
                    match UserData::insert(crate::structure::database::user_data::ActiveModel {
                        user_id: Set(id.clone()),
                        username: Set(member.user.name),
                        added_at: Set(Utc::now().naive_utc()),
                        ..Default::default()
                    })
                    .on_conflict(
                        OnConflict::column(crate::structure::database::user_data::Column::UserId)
                            .update_column(crate::structure::database::user_data::Column::Username)
                            .to_owned(),
                    )
                    .exec(&connection)
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => error!("Failed to insert user data. {}", e),
                    };
                    continue;
                }
                average_colors.push((color, pfp_url_old, image_old));
                continue;
            }
            _ => {
                let (average_color, image): (String, String) =
                    calculate_user_color(member.clone()).await?;
                average_colors.push((average_color.clone(), pfp_url.clone(), image.clone()));
                UserColor::insert(ActiveModel {
                    user_id: Set(id.clone()),
                    profile_picture_url: Set(pfp_url.clone()),
                    color: Set(average_color.clone()),
                    images: Set(image.clone()),
                    ..Default::default()
                })
                .on_conflict(
                    // update url, color, images and calculated_at
                    OnConflict::column(Column::UserId)
                        .update_column(Column::Color)
                        .update_column(Column::ProfilePictureUrl)
                        .update_column(Column::Images)
                        .update_column(Column::CalculatedAt)
                        .to_owned(),
                )
                .exec(&connection)
                .await?;
                match UserData::insert(crate::structure::database::user_data::ActiveModel {
                    user_id: Set(id.clone()),
                    username: Set(member.user.name),
                    added_at: Set(Utc::now().naive_utc()),
                    ..Default::default()
                })
                .on_conflict(
                    OnConflict::column(crate::structure::database::user_data::Column::UserId)
                        .update_column(crate::structure::database::user_data::Column::Username)
                        .to_owned(),
                )
                .exec(&connection)
                .await
                {
                    Ok(_) => {}
                    Err(e) => error!("Failed to insert user data. {}", e),
                };
                continue;
            }
        }
    }

    Ok(average_colors)
}

/// Calculates the average color of a user's profile picture.
///
/// This function retrieves the user's profile picture URL, downloads the image, and calculates the average color.
/// The image is converted to rgba8 format to ensure consistent color types.
/// The function uses rayon for CPU multithreading to calculate the total red, green, and blue values in the image.
/// The average color is then calculated by dividing the total by the number of pixels.
/// The function also encodes the image in base64 format.
///
/// # Arguments
///
/// * `member` - A `Member` object representing the user to calculate the color for.
///
/// # Returns
///
/// * `Result<(String, String), AppError>` - On success, the function returns `Ok((String, String))`, where the first element is the average color in hexadecimal format, and the second element is the base64 encoded image.
///   If the function fails to calculate the color, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if there's a problem retrieving the user's profile picture, downloading the image, or calculating the color.
async fn calculate_user_color(member: Member) -> Result<(String, String), Box<dyn Error>> {
    let pfp_url = member.user.face().replace("?size=1024", "?size=64");

    let img = get_image_from_url(pfp_url).await?;

    // convert to rgba8 so every image use the same color type.
    let img = img.to_rgba8();

    // Fallback to CPU multithreading with rayon
    let (r_total, g_total, b_total) = img
        .enumerate_pixels()
        .par_bridge()
        .map(|(_, _, pixel)| (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32))
        .reduce(
            || (0, 0, 0),
            |(r1, g1, b1), (r2, g2, b2)| (r1 + r2, g1 + g2, b1 + b2),
        );

    debug!("R: {}, G: {}, B: {}", r_total, g_total, b_total);

    // Calculate the average color by dividing the sum by the total number of pixels
    let num_pixels = img.width() * img.height();
    let r_avg = r_total / num_pixels;
    let g_avg = g_total / num_pixels;
    let b_avg = b_total / num_pixels;

    let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);
    debug!("{}", average_color);

    let mut image_data: Vec<u8> = Vec::new();
    PngEncoder::new(&mut image_data).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        ExtendedColorType::Rgba8,
    )?;

    let base64_image = general_purpose::STANDARD.encode(image_data.clone());
    let image = format!("data:image/png;base64,{}", base64_image);
    // Return the average color
    Ok((average_color, image))
}

/// Fetches an image from a given URL and decodes it into a `DynamicImage`.
///
/// This function performs the following steps:
/// 1. Sends a GET request to the provided URL to fetch the image data.
/// 2. Converts the response into bytes.
/// 3. Creates a new `ImageReader` with the image data.
/// 4. Decodes the image data into a `DynamicImage`.
///
/// # Arguments
///
/// * `url` - A string representing the URL of the image to fetch.
///
/// # Returns
///
/// * `Result<DynamicImage, AppError>` - On success, the function returns `Ok(DynamicImage)`.
///   If the function fails to fetch or decode the image, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if there's a problem fetching the image from the URL,
/// converting the response into bytes, or decoding the image data.
pub async fn get_image_from_url(url: String) -> Result<DynamicImage, Box<dyn Error>> {
    // Fetch the image data
    let resp = reqwest::get(&url).await?.bytes().await?;

    // Decode the image data
    let img = ImageReader::new(Cursor::new(resp))
        .with_guessed_format()?
        .decode()?;

    Ok(img)
}

/// Manages the color calculation for each member of the guilds.
///
/// This function performs the following steps:
/// 1. Initializes an empty vector to store the members of the guilds.
/// 2. Iterates over each guild, retrieving the members and appending them to the vector.
/// 3. Calls the `calculate_users_color` function to calculate the color for each member.
/// 4. If there's an error during the color calculation, it logs the error.
///
/// # Arguments
///
/// * `guilds` - A reference to a vector of `GuildId` objects representing the guilds to manage the color for.
/// * `ctx_clone` - A reference to the `Context` object, which is used to interact with Discord's API.
///
/// # Errors
///
/// This function will log an error if there's a problem calculating the color for any member.
pub async fn color_management(
    guilds: &Vec<GuildId>,
    ctx_clone: &Context,
    user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
    db_config: BotConfigDetails,
) {
    let mut futures = FuturesUnordered::new();
    for guild in guilds {
        let guild_id = guild.to_string();
        debug!(guild_id);

        let ctx_clone = ctx_clone.clone();
        let guild = *guild;
        let future = get_member(ctx_clone, guild);
        futures.push(future);
    }
    let mut members = Vec::new();
    while let Some(mut result) = futures.next().await {
        let guild_id = match result.first() {
            Some(member) => member.guild_id.to_string(),
            None => String::from(""),
        };
        debug!("{}: {}", guild_id, result.len());
        members.append(&mut result);
    }
    match calculate_users_color(
        members.into_iter().collect(),
        user_blacklist_server_image,
        db_config,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => error!("{:?}", e),
    };
}

/// Fetches all members of a given guild.
///
/// This function fetches members of a guild in batches of 1000 (the maximum allowed by Discord's API).
/// It keeps fetching until the number of members fetched is less than 1000, indicating that all members have been fetched.
///
/// # Arguments
///
/// * `ctx_clone` - A reference to the `Context` object, which is used to interact with Discord's API.
/// * `guild` - A reference to a `GuildId` object representing the guild to fetch members from.
///
/// # Returns
///
/// * `Vec<Member>` - A vector of `Member` objects representing the members of the guild.
///
/// # Panics
///
/// This function will panic if there's a problem fetching the members from the guild.
pub async fn get_member(ctx_clone: Context, guild: GuildId) -> Vec<Member> {
    let mut i = 0;
    let mut members_temp_out: Vec<Member> = Vec::new();
    while members_temp_out.len() == (1000 * i) {
        let mut members_temp_in = if i == 0 {
            match guild.members(&ctx_clone.http, Some(1000), None).await {
                Ok(members) => members,
                Err(e) => {
                    error!("{:?}", e);
                    break;
                }
            }
        } else {
            let user: UserId = match members_temp_out.last() {
                Some(member) => member.user.id,
                None => break,
            };
            match guild.members(&ctx_clone.http, Some(1000), Some(user)).await {
                Ok(members) => members,
                Err(e) => {
                    error!("{:?}", e);
                    break;
                }
            }
        };
        i += 1;
        members_temp_out.append(&mut members_temp_in);
    }
    members_temp_out
}
