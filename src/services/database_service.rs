use sea_orm::{Database, DbConn, DbErr};

#[async_trait::async_trait]
pub trait DatabaseProvider {
    async fn connect(&self, url: String) -> Result<DatabaseConnection>;
}

pub struct DatabaseService {
    
}

impl DatabaseService {
    pub fn new() -> Self {
        Self {

        }
    }
}

#[async_trait::async_trait]
impl DatabaseProvider for DatabaseService {
    async fn connect(&self, url: String) -> Result<DatabaseConnection> {
        DatabaseConnection::try_new(url).await
    }
}

pub struct DatabaseConnection {
    url: String,
    connection: DbConn,
}

impl DatabaseConnection {
    pub async fn try_new(url: String) -> Result<Self> {
        let connection = Database::connect(&url).await?;

        Ok(Self {
            url,
            connection,
        })
    }
}

type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error(transparent)]
    DbError(#[from] DbErr)
}