// External uses
use serde::Deserialize;
// Local uses
use crate::envy_load;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct UserRegistrationConfig {
    user_admin_key: String,
}

impl UserRegistrationConfig {
    pub fn from_env() -> Self {
        envy_load!("user_registration", "USER_REGISTRATION_")
    }

    pub fn user_admin_key(&self) -> String {
        self.user_admin_key.clone()
    }
}