#[mashin_sdk::resource]
pub mod user {
    use mashin_sdk::{
        ext::parking_lot::Mutex, ProviderState, ResourceDefault, ResourceDiff, Result,
    };
    use mysql::prelude::*;
    use std::sync::Arc;

    #[mashin::config]
    pub struct Config {
        /// The source host of the user.
        ///
        /// @default "localhost"
        pub(crate) host: Option<String>,
        /// The password for the user. This must be provided in plain text, so the data source for it must be secured.
        ///
        /// An hash of the provided password is stored in state.
        ///
        /// @example "myp@ssw0rd"
        pub(crate) plaintext_password: Option<String>,
        /// TLS-Option for the `CREATE USER` or `ALTER USER` statement.
        /// The value is suffixed to `REQUIRE`. A value of 'SSL' will generate a `CREATE USER ... REQUIRE SSL` statement.
        /// See the MYSQL CREATE USER documentation for more.
        /// Ignored if MySQL version is under 5.7.0.
        ///
        /// @example "SSL"
        pub(crate) tls_option: Option<String>,
    }

    /// The `user` resource creates and manages a user on a MySQL server.
    #[mashin::resource]
    pub struct Resource {}

    #[mashin::calls]
    impl mashin_sdk::Resource for Resource {
        async fn get(&mut self, _provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            Ok(())
        }

        async fn create(&mut self, provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };

            let user_name = self.name();
            let host = self.config().host.as_deref().unwrap_or("localhost");
            let plaintext_password = self
                .config()
                .plaintext_password
                .as_deref()
                .unwrap_or_default();

            let mut query =
                format!("CREATE USER '{user_name}'@'{host}' IDENTIFIED BY '{plaintext_password}'",);

            if let Some(tls_option) = self.config().tls_option.as_deref() {
                // > 5.7.0
                let (major, minor, patch) = mysql_conn.server_version();
                if major >= 5 && minor >= 7 && patch > 0 {
                    query = format!("{query} REQUIRE {tls_option}");
                }
            }

            mysql_conn.query_drop(format!("{query};"))?;

            self.get(provider_state).await
        }

        async fn delete(&mut self, provider_state: Arc<Mutex<ProviderState>>) -> Result<()> {
            let mut mysql_conn = {
                let state = provider_state.lock();
                state.borrow::<mysql::Pool>().get_conn()?
            };

            let user_name = self.name();
            mysql_conn
                .query_drop(format!("DROP USER {user_name};"))
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
            let user_name = self.name();
            let host = self.config().host.as_deref().unwrap_or_default();
            if diff.has_change("config.plaintextPassword") {
                let password = self
                    .config()
                    .plaintext_password
                    .as_deref()
                    .unwrap_or_default();
                let (major, minor, patch) = mysql_conn.server_version();
                if major >= 5 && minor >= 7 && patch >= 6 {
                    mysql_conn.query_drop(format!(
                        "SET PASSWORD FOR '{user_name}'@'{host}' = PASSWORD('{password}');"
                    ))?
                } else {
                    mysql_conn.query_drop(format!(
                        "ALTER USER '{user_name}'@'{host}' IDENTIFIED BY '{password}';"
                    ))?
                }
            }

            self.get(provider_state).await
        }
    }
}
