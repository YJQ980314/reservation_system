use std::ops::Deref;
use abi::Config;
use sqlx_db_test::TestDb;

#[derive(Debug)]
pub struct TestConfig {
    pub config: Config,
    #[allow(dead_code)]
    tdb: TestDb,
}

impl Deref for TestConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl TestConfig {
    #[allow(dead_code)]
    
    pub fn new() -> Self {
        let mut config = Config::load("fixtures/config.yml").unwrap();
        
        let tdb = TestDb::new(
            &config.db.host,
            config.db.port,
            &config.db.user,
            &config.db.password,
            "../migrations",
        );

        config.db.dbname = tdb.dbname.clone();
        Self { config, tdb }
    }

    pub fn with_server_port(port: u16) -> Self {
        let mut config = TestConfig::default();
        config.config.server.port = port;
        config
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}