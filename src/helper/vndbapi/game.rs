use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub violence: f64,
    pub url: String,
    pub sexual: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Staff {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Titles {
    pub title: String,
    pub lang: String,
    pub official: bool,
    pub main: bool,
    pub latin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tags {
    pub name: String,
    pub id: String,
    pub rating: f64,
    pub spoiler: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Developers {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Results {
    pub olang: String,
    pub va: Vec<Va>,
    pub released: Option<String>,
    pub id: String,
    pub image: Option<Image>,
    pub staff: Vec<Staff>,
    pub rating: Option<f64>,
    pub length_minutes: Option<f64>, // Kept as Option since it might be null
    pub platforms: Vec<String>,
    pub title: String,
    pub average: Option<f64>,
    pub titles: Vec<Titles>,
    pub votecount: f64,
    pub languages: Vec<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<Tags>,
    pub description: Option<String>,
    pub devstatus: i32,
    pub developers: Vec<Developers>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VN {
    pub results: Vec<Results>,
    pub more: bool,
}

pub async fn get_vn(
    value: String,
) -> Result<VN, crate::helper::error_management::error_enum::AppError> {
    let value = value.to_lowercase();
    let value = value.trim();
    let start_with_v = value.starts_with('v');
    let is_number = value.chars().skip(1).all(|c| c.is_numeric());
    let json = if start_with_v && is_number {
        (r#"{
    		"filters": ["id", "=",""#.to_owned() + value + r#""],
    		"fields": "id,title,alttitle,titles.lang,titles.title,titles.latin,titles.official,titles.main, aliases,olang,devstatus,released,languages,platforms,image.url,image.sexual,image.violence,length_minutes,description,average,rating,votecount,tags.rating,tags.spoiler,tags.name,developers.name,staff.name,staff.role,va.character.name"
		}"#).to_string()
    } else {
        (r#"{
    		"filters": ["search", "=",""#.to_owned() + value + r#""],
    		"fields": "id,title,alttitle,titles.lang,titles.title,titles.latin,titles.official,titles.main, aliases,olang,devstatus,released,languages,platforms,image.url,image.sexual,image.violence,length_minutes,description,average,rating,votecount,tags.rating,tags.spoiler,tags.name,developers.name,staff.name,staff.role,va.character.name"
		}"#).to_string()
    };
    let path = "/vn".to_string();
    let response =
        crate::helper::vndbapi::common::do_request_cached_with_json(path.clone(), json.to_string())
            .await?;
    let response: VN = serde_json::from_str(&response).map_err(|e| {
        crate::helper::error_management::error_enum::AppError {
            message: format!("Error while parsing response: '{}'", e),
            error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
            error_response_type:
                crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
        }
    })?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub id: String,

    pub name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Va {
    pub character: Character,
}
