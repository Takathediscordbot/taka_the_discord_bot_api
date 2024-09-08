#![cfg(feature = "database")]

#![allow(unused)]

use std::{fs::File, io::Write};


use anyhow::anyhow;
use sqlx::FromRow;

use crate::api::api_v1::{models::silly_command::{RawSillyCommandData, Usages, SillyCommandType, SillyCommandData, self, CommandUsage, CommandId, RandomImage, CommandTextId, CommandSelfActionTextId, CommandImageId, CommandSelfActionImageId}, ApiV1State};







pub struct SillyCommandPDO;
impl SillyCommandPDO {
    
    pub async fn fetch_silly_commands(context: &ApiV1State<'_>) -> Vec<SillyCommandData> {
        let Ok(silly_commands) = sqlx::query_as::<_, RawSillyCommandData>(include_str!("../sql/silly_commands/fetch_silly_commands.sql"))
        .fetch_all(&context.sql_connection)
        .await else {
            return vec![]
        };

        silly_commands
            .into_iter()
            .filter_map(RawSillyCommandData::into_silly_command_data)
            .collect()
    }

    pub async fn fetch_command_usage(
        context: &ApiV1State<'_>,
        command: i32,
        author: u64,
        user: u64,
    ) -> Option<CommandUsage> {
        let record = sqlx::query_as::<_, CommandUsage>(include_str!("../sql/silly_commands/fetch_command_usage.sql"))
        .bind(author.to_string())
        .bind(user.to_string())
        .bind(command)
        .fetch_one(&context.sql_connection)
        .await
        .ok();

        record
    }

    pub async fn increment_command_usage(
        context: &ApiV1State<'_>,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<i32> {
        Ok(sqlx::query_as::<_, Usages>(include_str!("../sql/silly_commands/increment_command_usage.sql"))
            .bind(author.to_string())
            .bind(user.to_string()) 
            .bind(command)
            .fetch_one(&context.sql_connection)
            .await.map(|usage| usage.usages)?
        )
    }

    pub async fn create_command_usage(
        context: &ApiV1State<'_>,
        command: i32,
        author: u64,
        user: u64,
    ) -> anyhow::Result<i32> {
        Ok(sqlx::query_as::<_, CommandUsage>(include_str!("../sql/silly_commands/create_command_usage.sql"))
        .bind(command) 
        .bind(author.to_string())
        .bind(user.to_string())
        .fetch_one(&context.sql_connection)
        .await.map(|usage| usage.usages)?)
    }

    pub async fn create_command(
        context: &ApiV1State<'_>,
        command_name: &str,
        description: &str,
        footer_text: &str,
        command_type: SillyCommandType,
    ) -> anyhow::Result<i32> {
        let id = sqlx::query_as::<_, CommandId>(include_str!("../sql/silly_commands/create_command.sql"))
        .bind(command_name)
        .bind(description)
        .bind(command_type as i32)
        .bind(footer_text)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command)
    }

    pub async fn add_preference(
        context: &ApiV1State<'_>,
        preference: &str,
        command: &str
    )
    -> anyhow::Result<()> {
        sqlx::query(include_str!("../sql/silly_commands/add_preference.sql"))
        .bind(preference)
        .bind(command)
        .execute(&context.sql_connection)
        .await?;

        Ok(())
    }
    

    pub async fn fetch_silly_command_by_name(
        context: &ApiV1State<'_>,
        name: &str,
    ) -> Option<SillyCommandData> {

        sqlx::query_as::<_, RawSillyCommandData>(include_str!(
        "../sql/silly_commands/fetch_silly_command_by_name.sql"))
        .bind(name)
        .fetch_optional(&context.sql_connection)
        .await
        .ok()
        .flatten()
        .and_then(|silly_command| silly_command.into_silly_command_data())
        
    }

    pub async fn fetch_random_silly_image_by_name_and_preference(
        context: &ApiV1State<'_>,
        command: i32,
        preference: &str
    ) -> anyhow::Result<String> {
        let result =
        
        sqlx::query_as::<_, RandomImage>(include_str!(
            "../sql/silly_commands/fetch_random_silly_image_by_name_and_preference.sql"))
        .bind(command)
        .bind( preference)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(result.image)
    }


    pub async fn add_text(
        context: &ApiV1State<'_>,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;
        
        let id = sqlx::query_as::<_, CommandTextId>(include_str!("../sql/silly_commands/add_text.sql"))
        .bind(command.id_silly_command)
        .bind(content)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_text)
    }

    pub async fn add_text_author(
        context: &ApiV1State<'_>,
        command_name: &str,
        content: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let id = CommandSelfActionTextId::from_row(&sqlx::query(include_str!("../sql/silly_commands/add_author_text.sql"))
        .bind(command.id_silly_command)
        .bind(content)
        .fetch_one(&context.sql_connection)
        .await?)?;

        Ok(id.id_silly_command_self_action_text)
    }

    pub async fn add_image(
        context: &ApiV1State<'_>,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
        preference: Option<String>
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        if matches!(command.command_type, SillyCommandType::AuthorOnly) {
            return Self::add_image_author(context, command_name, image, extension).await;
        }

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;
        let preference = preference.unwrap_or("ALL".to_string());

        out.write_all(&image[..])?;

        let id = sqlx::query_as::<_, CommandImageId>(include_str!("../sql/silly_commands/add_image.sql"))
        .bind(command.id_silly_command)
        .bind(file_path) 
        .bind(preference)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_images)
    }


        

    pub async fn add_image_author(
        context: &ApiV1State<'_>,
        command_name: &str,
        image: Vec<u8>,
        extension: &str,
    ) -> anyhow::Result<i32> {
        let command = Self::fetch_silly_command_by_name(&context, command_name)
            .await
            .ok_or(anyhow!("Couldn't find command!"))?;

        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = format!("./assets/{file_name}.{extension}");
        let mut out = File::create(&file_path)?;

        out.write_all(&image[..])?;

        let id = sqlx::query_as::<_, CommandSelfActionImageId>(include_str!("../sql/silly_commands/add_image_author.sql"))
        .bind(command.id_silly_command)
        .bind(file_path)
        .fetch_one(&context.sql_connection)
        .await?;

        Ok(id.id_silly_command_self_action)
    }
}


