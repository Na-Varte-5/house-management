use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<MysqlConnection>>;
