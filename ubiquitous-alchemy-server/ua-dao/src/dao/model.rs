use sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions, Database, MySql, Pool, Postgres};

pub use crate::interface::UaQuery;
pub use crate::interface::UaSchema;
use crate::util::DataEnum;
use crate::util::DbQueryResult;
use crate::DaoError;
use crate::QueryResult;

pub type DaoPG = Dao<Postgres>;
pub type DaoMY = Dao<MySql>;

pub struct Dao<T: Database> {
    pub pool: Pool<T>,
}

impl<T: Database> Clone for Dao<T> {
    fn clone(&self) -> Self {
        Dao {
            pool: self.pool.clone(),
        }
    }
}

impl Dao<Postgres> {
    pub async fn new(uri: &str, max_connections: u32) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(uri)
            .await
            .unwrap();

        Dao { pool }
    }
}

impl Dao<MySql> {
    pub async fn new(uri: &str, max_connections: u32) -> Self {
        let pool = MySqlPoolOptions::new()
            .max_connections(max_connections)
            .connect(uri)
            .await
            .unwrap();

        Dao { pool }
    }
}

#[derive(Clone)]
pub enum DaoOptions {
    PG(DaoPG),
    MY(DaoMY),
}

impl DaoOptions {
    pub async fn exec(&self, query: &str) -> Result<Box<dyn QueryResult>, DaoError> {
        match self {
            DaoOptions::PG(p) => match sqlx::query(query).execute(&p.pool).await {
                Ok(r) => Ok(Box::new(DbQueryResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: None,
                })),
                Err(e) => Err(DaoError::from(e)),
            },
            DaoOptions::MY(p) => match sqlx::query(query).execute(&p.pool).await {
                Ok(r) => Ok(Box::new(DbQueryResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: Some(r.last_insert_id()),
                })),
                Err(e) => Err(DaoError::from(e)),
            },
        }
    }

    pub async fn seq_exec(
        &self,
        seq_query: &Vec<String>,
    ) -> Result<Box<dyn QueryResult>, DaoError> {
        match self {
            DaoOptions::PG(p) => {
                let mut tx = match p.pool.begin().await {
                    Ok(t) => t,
                    Err(e) => return Err(DaoError::from(e)),
                };

                for query in seq_query {
                    if let Err(e) = sqlx::query(query).execute(&mut tx).await {
                        return Err(DaoError::from(e));
                    }
                }

                match tx.commit().await {
                    Ok(_) => Ok(Box::new(DataEnum::Bool(true))),
                    Err(e) => Err(DaoError::from(e)),
                }
            }
            DaoOptions::MY(p) => {
                let mut tx = match p.pool.begin().await {
                    Ok(t) => t,
                    Err(e) => return Err(DaoError::from(e)),
                };

                for query in seq_query {
                    if let Err(e) = sqlx::query(query).execute(&mut tx).await {
                        return Err(DaoError::from(e));
                    }
                }

                match tx.commit().await {
                    Ok(_) => Ok(Box::new(DataEnum::Bool(true))),
                    Err(e) => Err(DaoError::from(e)),
                }
            }
        }
    }
}
