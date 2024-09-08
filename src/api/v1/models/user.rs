

use chrono::prelude::*;
use serde::{Deserialize, Serialize};


#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub password_rev: uuid::Uuid
}

#[allow(unused)]
impl User {
    pub fn has_role<'a>(&'a self, roles: &[&'a str]) -> bool {
        roles.contains(&self.role.as_str())
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(&["admin"])
    }

    pub fn is_user(&self) -> bool {
        self.has_role(&["user"])
    }

    pub fn is_moderator(&self) -> bool {
        self.has_role(&["moderator"])
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }


}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordSchema {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub email: String,
    pub password: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUser {
    pub id: String,
    pub password_rev: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserData {
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
    pub id: String
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ForceUpdateUser {
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
    pub id: String,
}