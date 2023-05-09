#[mashin_sdk::resource]
pub mod database {
    use mashin_sdk::{
        ext::parking_lot::Mutex, ProviderState, ResourceDefault, ResourceDiff, Result,
    };
    use mysql::prelude::*;
    use regex::Regex;
    use std::sync::Arc;

    const DEFAULT_CHARACTER_SET: &'static str = "utf8mb4";

    #[mashin::config]
    pub struct Config {
        /// The default character set to use when a table is created without specifying an explicit character set.
        ///
        /// @example "utf8"
        pub(crate) default_character_set: Option<String>,
    }

    /// The `database` resource creates and manages a database on a MySQL server.
    #[mashin::resource]
    pub struct Resource {
        /// The default_character_set of the database.
        ///
        /// @example "utf8"
        pub(crate) default_character_set: Option<String>,
        /// The default_collation of the database.
        ///
        /// @example "utf8_general_ci"
        pub(crate) default_collation: Option<String>,
    }

    #[mashin::calls]
    impl mashin_sdk::Resource for Resource {
        async fn get(&mut self, provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };

            let db_name: &str = self.name();
            if mysql_conn.as_mut().select_db(db_name) {
                let regex_charset =
                    Regex::new(r".+DEFAULT CHARACTER SET[[:blank:]]+([[:word:]]+)")?;
                let regex_collate = Regex::new(r".+COLLATE[[:blank:]]+([[:word:]]+)")?;
                mysql_conn.query_map(
                    format!("SHOW CREATE DATABASE {db_name};"),
                    |(_, create_database): (String, String)| {
                        let result = regex_charset.captures(&create_database);
                        if let Some(default_charset) = result {
                            if let Some(default_charset) = default_charset
                                .get(1)
                                .map(|char_set: regex::Match| char_set.as_str())
                            {
                                self.default_character_set = Some(default_charset.to_string());
                            };
                        }

                        let result = regex_collate.captures(&create_database);
                        if let Some(default_collation) = result {
                            if let Some(default_collation) = default_collation
                                .get(1)
                                .map(|char_set: regex::Match| char_set.as_str())
                            {
                                self.default_collation = Some(default_collation.to_string());
                            };
                        }
                    },
                )?;
            }

            Ok(())
        }

        async fn create(&mut self, provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };

            let db_name = self.name();
            mysql_conn.query_drop(format!("CREATE DATABASE {db_name};"))?;

            self.get(provider_state).await
        }

        async fn delete(&mut self, provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };

            let db_name = self.name();
            mysql_conn
                .query_drop(format!("DROP DATABASE {db_name};"))
                .map_err(Into::into)
        }

        async fn update(
            &mut self,
            provider_state: Arc<Mutex<ProviderState>>,
            diff: &ResourceDiff,
        ) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };
            let db_name = self.name();
            if diff.has_change("config.defaultCharacterSet") {
                let default_character_set = self
                    .config()
                    .default_character_set
                    .as_deref()
                    .unwrap_or(DEFAULT_CHARACTER_SET);

                mysql_conn.query_drop(format!(
                    "ALTER DATABASE {db_name} CHARACTER SET {default_character_set};"
                ))?
            }

            self.get(provider_state).await
        }
    }
}
