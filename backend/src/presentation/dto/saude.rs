use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SaudeResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub db: &'static str,
}

impl SaudeResponse {
    pub const fn ok() -> Self {
        Self {
            status: "ok",
            service: "accurit-backend",
            db: "ok",
        }
    }

    pub const fn indisponivel() -> Self {
        Self {
            status: "unavailable",
            service: "accurit-backend",
            db: "unavailable",
        }
    }
}
