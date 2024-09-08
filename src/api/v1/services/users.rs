#![cfg(feature = "database")]

// create service to handle user functions

use std::time::SystemTime;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use chrono::{DateTime, Utc};
use rand_core::OsRng;

use crate::api::api_v1::{ApiV1State, models::user::{User, RegisterUserSchema, LoginUserSchema, CreateUser, UpdateUserData}};

pub struct UserPDO;

impl UserPDO {
    // update user password
    pub async fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    }

    // get all users
    pub async fn fetch_users(context: &ApiV1State<'_>) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(include_str!("../sql/users/fetch_users.sql"))
            .fetch_all(&context.sql_connection)
            .await?;
        Ok(users)
    }

    // get user by id
    pub async fn fetch_user_by_id(context: &ApiV1State<'_>, user_id: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(include_str!("../sql/users/fetch_user_by_id.sql"))
            .bind(user_id)
            .fetch_optional(&context.sql_connection)
            .await?;
        Ok(user)
    }

    // verify user password 
    pub async fn verify_user_password(context: &ApiV1State<'_>, user: &User, password: &str) -> bool {

        let result = match PasswordHash::new(&user.password) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_or(false, |_| true),
            Err(_) => false,
        };
        
        result
    }

    // get user by email
    pub async fn fetch_user_by_email(context: &ApiV1State<'_>, email: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(include_str!("../sql/users/fetch_user_by_email.sql"))
            .bind(email)
            .fetch_optional(&context.sql_connection)
            .await?;
        Ok(user)
    }

    // login user
    pub async fn login_user(context: &ApiV1State<'_>, LoginUserSchema {email, password}: &LoginUserSchema) -> anyhow::Result<User> {
        let user = Self::fetch_user_by_email(context, &email).await?.ok_or( anyhow::anyhow!("Invald email or password"))?;
        let password_verified = Self::verify_user_password(context, &user, password).await;
        if !password_verified {
            return Err(anyhow::anyhow!("Invalid password"));
        }
        Ok(user)
    }


    // register user
    pub async fn register_user(context: &ApiV1State<'_>, RegisterUserSchema {
        name,
        email,
        password,
    }: &RegisterUserSchema, role: Option<&str>) -> anyhow::Result<User> {
        let user = Self::create_user(context, &CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: password.clone(),
            role: role.unwrap_or("user").to_string(),
            verified: false
        }).await?;
        Ok(user)
    }

    // create user
    pub async fn create_user(context: &ApiV1State<'_>, CreateUser {name, email, password, role, verified, ..}: &CreateUser) -> anyhow::Result<User> {
        // check if user exists
        let user_exists = Self::user_exists(context, email).await?;
        if user_exists {
            return Err(anyhow::anyhow!("User already exists"));
        }
        
        let password = Self::hash_password(password).await.map_err(|e| anyhow::anyhow!("Failed to hash password\n{e}"))?;
        Ok(sqlx::query_as::<_, User>(include_str!("../sql/users/create_user.sql"))
            .bind(name)
            .bind(email)
            .bind(password)
            .bind(role)
            .bind(verified)
            .fetch_one(&context.sql_connection)
            .await?)
    }

    // user_exists
    pub async fn user_exists(context: &ApiV1State<'_>, email: &str) -> anyhow::Result<bool> {
        let user = sqlx::query_as::<_, User>(include_str!("../sql/users/user_exists.sql"))
            .bind(email)
            .fetch_optional(&context.sql_connection)
            .await?;
        Ok(user.is_some())
    }

    // delete user
    pub async fn delete_user(context: &ApiV1State<'_>, user_id: &str) -> anyhow::Result<()> {
        sqlx::query(include_str!("../sql/users/delete_user.sql"))
            .bind(user_id)
            .execute(&context.sql_connection)
            .await?;
        Ok(())
    }

    pub async fn update_user_password(context: &ApiV1State<'_>, user_id: &str, password: &str) -> anyhow::Result<()> {
        let password = Self::hash_password(password).await.map_err(|e| anyhow::anyhow!("Failed to hash password\n{e}"))?;
        sqlx::query(include_str!("../sql/users/update_user_password.sql"))
            .bind(user_id)
            .bind(password)
            .execute(&context.sql_connection)
            .await?;
        Ok(())
    }

    pub async fn update_user(context: &ApiV1State<'_>, UpdateUserData {name, email, role, verified, id, ..}: &UpdateUserData) -> anyhow::Result<User> {
        Ok(sqlx::query_as::<_, User>(include_str!("../sql/users/update_user.sql"))
            .bind(name)
            .bind(email)
            .bind(role)
            .bind(verified)
            .bind(DateTime::<Utc>::from(SystemTime::now()))
            .bind(id)
            .fetch_one(&context.sql_connection)
            .await?)
    }
}

