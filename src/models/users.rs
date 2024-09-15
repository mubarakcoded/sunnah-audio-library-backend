use serde::{Deserialize, Serialize};
use uuid::Uuid;



#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: usize,
    pub exp: usize,
    pub role: String
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, sqlx::Type, PartialEq, Eq)]
// #[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum Role {
    Admin,
    Manager,
    Viewer,
    ServiceAdmin,
    ServiceManager,
    ServiceViewer,
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(Role::Admin),
            "Manager" => Ok(Role::Manager),
            "Viewer" => Ok(Role::Viewer),
            "ServiceAdmin" => Ok(Role::ServiceAdmin),
            "ServiceManager" => Ok(Role::ServiceManager),
            "ServiceViewer" => Ok(Role::ServiceViewer),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl std::string::ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Admin => "Admin".to_string(),
            Role::Manager => "Manager".to_string(),
            Role::Viewer => "Viewer".to_string(),
            Role::ServiceAdmin => "ServiceAdmin".to_string(),
            Role::ServiceManager => "ServiceManager".to_string(),
            Role::ServiceViewer => "ServiceViewer".to_string(),
        }
    }
}
