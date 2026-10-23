use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EscritorioId(Uuid);

impl EscritorioId {
    pub fn novo() -> Self {
        Self(Uuid::now_v7())
    }

    pub const fn de_uuid(valor: Uuid) -> Self {
        Self(valor)
    }

    pub const fn como_uuid(self) -> Uuid {
        self.0
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for EscritorioId {
    fn from(valor: Uuid) -> Self {
        Self(valor)
    }
}

impl From<EscritorioId> for Uuid {
    fn from(valor: EscritorioId) -> Self {
        valor.0
    }
}
