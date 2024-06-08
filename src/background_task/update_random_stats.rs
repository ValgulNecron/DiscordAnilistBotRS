use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media::MediaDataId;
use crate::structure::run::anilist::site_statistic_anime::{AnimeStat, AnimeStatVariables};
use crate::structure::run::anilist::site_statistic_manga::{MangaStat, MangaStatVariables};
use cynic::{GraphQlResponse, QueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RandomStat {
    pub anime_last_page: i32,
    pub manga_last_page: i32,
}

pub async fn update_random_stats_launcher() {
    info!("Starting random stats update");
    loop {
        let _ = update_random_stats().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(86_400)).await;
    }
}

pub async fn update_random_stats() -> Result<RandomStat, AppError> {
    // try to load random stats from a json file
    let mut random_stats: RandomStat = match std::fs::read_to_string("random_stats.json") {
        Ok(stats) => serde_json::from_str(&stats).map_err(|e| {
            AppError::new(
                format!("There was an error deserializing the random stats {}", e),
                ErrorType::File,
                ErrorResponseType::Unknown,
            )
        })?,
        Err(_) => {
            let random_stats = RandomStat {
                anime_last_page: 1796,
                manga_last_page: 1796,
            };
            random_stats
        }
    };
    random_stats = update_random(random_stats).await?;
    // write random stats to a json file
    let random_stats_json = serde_json::to_string(&random_stats).map_err(|e| {
        AppError::new(
            format!("There was an error serializing the random stats {}", e),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    std::fs::write("random_stats.json", random_stats_json).map_err(|e| {
        AppError::new(
            format!(
                "There was an error writing the random stats to a file {}",
                e
            ),
            ErrorType::File,
            ErrorResponseType::Unknown,
        )
    })?;
    Ok(random_stats)
}

async fn update_random(mut random_stats: RandomStat) -> Result<RandomStat, AppError> {
    let mut has_next_page = true;
    while has_next_page {
        let anime_page = random_stats.anime_last_page;
        let manga_page = random_stats.manga_last_page;
        let var = AnimeStatVariables {
            page: Some(anime_page),
        };
        let operation = AnimeStat::build(var);
        let data: GraphQlResponse<AnimeStat> = match make_request_anilist(operation, false).await {
            Ok(data) => match data.json::<GraphQlResponse<AnimeStat>>().await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(?e);
                    return Err(AppError {
                        message: format!(
                            "Error retrieving character with ID: {} \n {}",
                            anime_page, e
                        ),
                        error_type: ErrorType::WebRequest,
                        error_response_type: ErrorResponseType::Message,
                    });
                }
            },
            Err(e) => {
                tracing::error!(?e);
                return Err(AppError {
                    message: format!(
                        "Error retrieving character with ID: {} \n {}",
                        anime_page, e
                    ),
                    error_type: ErrorType::WebRequest,
                    error_response_type: ErrorResponseType::Message,
                });
            }
        };
        has_next_page = data
            .data
            .unwrap()
            .site_statistics
            .unwrap()
            .manga
            .unwrap()
            .page_info
            .unwrap()
            .has_next_page
            .unwrap();
        if has_next_page {
            random_stats.anime_last_page = anime_page + 1;
            random_stats.manga_last_page = manga_page + 1;
        } else {
            random_stats.anime_last_page = anime_page - 1;
            random_stats.manga_last_page = manga_page - 1;
        }
    }

    let mut has_next_page = true;
    while has_next_page {
        let anime_page = random_stats.anime_last_page;
        let manga_page = random_stats.manga_last_page;
        let var = MangaStatVariables {
            page: Some(manga_page),
        };
        let operation = MangaStat::build(var);
        let data: GraphQlResponse<MangaStat> = match make_request_anilist(operation, false).await {
            Ok(data) => match data.json::<GraphQlResponse<MangaStat>>().await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(?e);
                    return Err(AppError {
                        message: format!(
                            "Error retrieving character with ID: {} \n {}",
                            anime_page, e
                        ),
                        error_type: ErrorType::WebRequest,
                        error_response_type: ErrorResponseType::Message,
                    });
                }
            },
            Err(e) => {
                tracing::error!(?e);
                return Err(AppError {
                    message: format!(
                        "Error retrieving character with ID: {} \n {}",
                        anime_page, e
                    ),
                    error_type: ErrorType::WebRequest,
                    error_response_type: ErrorResponseType::Message,
                });
            }
        };
        has_next_page = data
            .data
            .unwrap()
            .site_statistics
            .unwrap()
            .manga
            .unwrap()
            .page_info
            .unwrap()
            .has_next_page
            .unwrap();
        if has_next_page {
            random_stats.anime_last_page = anime_page + 1;
            random_stats.manga_last_page = manga_page + 1;
        } else {
            random_stats.anime_last_page = anime_page - 1;
            random_stats.manga_last_page = manga_page - 1;
        }
    }

    Ok(random_stats)
}
