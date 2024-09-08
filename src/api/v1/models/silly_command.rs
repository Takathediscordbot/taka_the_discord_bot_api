#![allow(unused)]

use serde::Deserialize;
use sqlx::FromRow;


#[derive(FromRow, Debug, Default)]
pub struct RawSillyCommandData {
    pub id_silly_command: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub footer_text: Option<String>,
    pub command_type: Option<i32>,
    pub self_texts: Option<Vec<String>>,
    pub self_images: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub preferences: Option<Vec<String>>,
    pub texts: Option<Vec<String>>,
    pub gender_attributes: Option<Vec<String>>
}

#[derive(Debug, serde::Serialize)]
pub struct SillyCommandData {
    pub id_silly_command: i32,
    pub name: String,
    pub description: String,
    pub footer_text: String,
    pub command_type: SillyCommandType,
    pub self_texts: Vec<String>,
    pub self_images: Vec<String>,
    pub preferences: Vec<String>,
    pub images: Vec<String>,
    pub texts: Vec<String>,
    pub gender_attributes: Vec<String>
}

impl RawSillyCommandData {
    pub fn into_silly_command_data(self) -> Option<SillyCommandData> {
        log::debug!("begining");
        
        let (id_silly_command, name, command_type, description) = match (
            self.id_silly_command,
            self.name,
            self.command_type,
            self.description,
        ) {
            (Some(id), Some(name), Some(command_type), Some(description)) => {
                (id, name, command_type, description)
            }
            _ => return None,
        };
        log::debug!("before footer command type");

        let command_type = match command_type.try_into().map(Some).unwrap_or(None) {
            Some(data) => data,
            _ => return None,
        };

        log::debug!("before footer text");

        let footer_text = match self.footer_text {
            Some(text) => text,
            _ => return None
        };

        log::debug!("after footer text");

        Some(SillyCommandData {
            id_silly_command,
            name,
            description,
            command_type,
            footer_text,
            self_texts: self
                .self_texts.unwrap_or_default(),
            self_images: self
                .self_images.unwrap_or_default(),
            preferences: self.preferences.unwrap_or_default(),
            images: self
                .images.unwrap_or_default(),
            texts: self.texts.unwrap_or_default(),
            gender_attributes: self.gender_attributes.unwrap_or_default()
        })
    }
}

#[repr(i32)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SillyCommandType {
    AuthorOnly = 1,
    SingleUser = 2,
}

impl TryFrom<i32> for SillyCommandType {
    type Error = ();
    fn try_from(value: i32) -> Result<SillyCommandType, ()> {
        match value {
            1 => Ok(Self::AuthorOnly),
            2 => Ok(Self::SingleUser),
            _ => Err(()),
        }
    }
}

impl SillyCommandData {}

#[derive(FromRow)]
pub struct Usages {
    pub usages: i32
}

#[derive(FromRow)]
pub struct CommandUsage {
    pub usages: i32
}

#[derive(FromRow)]
pub struct CommandUsageId {
    pub id_silly_command_usage: i32
}

#[derive(FromRow)]
pub struct CommandId {
    pub id_silly_command: i32
}

#[derive(FromRow)]
pub struct RandomImage {
    pub image: String
}

#[derive(FromRow)]
pub struct CommandTextId {
    pub id_silly_command_text: i32
}

#[derive(FromRow)]
pub struct CommandSelfActionTextId {
    pub id_silly_command_self_action_text: i32
}

#[derive(FromRow)]
pub struct CommandSelfActionImageId {
    pub id_silly_command_self_action: i32
}

#[derive(FromRow)]
pub struct CommandImageId {
    pub id_silly_command_images: i32
}

// struct AddImageAuthorRequest

#[derive(Deserialize)]
pub struct AddTextRequest {
    pub command_name: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct AddTextAuthorRequest {
    pub command_name: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct AddPreferenceRequest {
    pub command_name: String,
    pub preference: String,
}

#[derive(Deserialize)]
pub struct AddCommandRequest {
    pub command_name: String,
    pub description: String,
    pub footer_text: String,
    pub command_type: SillyCommandType,
}

#[derive(Deserialize)]
pub struct IncrementCommandUsage {
    pub command: i32,
    pub author: u64,
    pub user: u64,
}

#[derive(Deserialize)]
pub struct CreateCommandUsage {
    pub command: i32,
    pub author: u64,
    pub user: u64
}

#[derive(Deserialize)]
pub struct FetchCommandUsage {
    pub command: i32
}

#[derive(Deserialize)]
pub struct FetchRandomSillyImageByNameAndPreference {
    pub command: i32,
    pub preference: String
}

#[derive(Deserialize)]
pub struct FetchSillyCommandByName {
    pub name: String
}

