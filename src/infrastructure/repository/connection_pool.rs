use crate::utils::error::{ApplicationError, ApplicationResult};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::MysqlConnection;

pub struct MysqlConnectionPool {
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl MysqlConnectionPool {
    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");

        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(4)
            .build(manager)
            .expect("Failed to create pool");

        Self { pool }
    }

    pub fn get_connection(&self) -> ApplicationResult<PooledConnection<MysqlConnectionManager>> {
        let conn = self.pool.get().or_else(|_| {
            Err(ApplicationError::DataBaseError(
                "Failed to get DB connection.".into(),
            ))
        })?;
        Ok(conn)
    }
}
