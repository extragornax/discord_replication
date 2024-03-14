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
//
// pub fn with_db_access_manager(
//     pool: PgPool,
// ) -> impl Filter<Extract=(DBAccessManager, ), Error=warp::Rejection> + Clone {
//     warp::any()
//         .map(move || pool.clone())
//         .and_then(|pool: PgPool| async move {
//             match pool.get() {
//                 Ok(conn) => Ok(DBAccessManager::new(conn)),
//                 Err(err) => Err(reject::custom(AppError::new(
//                     format!("Error getting connection from pool: {}", err.to_string()).as_str(),
//                     ErrorType::Internal,
//                 ))),
//             }
//         })
// }

type PooledPg = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DBAccessManager {
    pub connection: PooledPg,
}

impl DBAccessManager {
    pub fn new(connection: PooledPg) -> DBAccessManager {
        DBAccessManager { connection }
    }
}
