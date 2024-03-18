use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_pg_pool(db_url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .max_size(5)
        .build(manager)
        .expect(&format!("Error connecting to {}", &db_url))
}

pub fn get_pg_pool_sized(db_url: &str, size: u32) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .max_size(size)
        .build(manager)
        .expect(&format!("Error connecting to {}", &db_url))
}

type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DBAccessManager {
    pub connection: PooledPg,
}

impl DBAccessManager {
    pub fn new(connection: PooledPg) -> DBAccessManager {
        DBAccessManager { connection }
    }
}
