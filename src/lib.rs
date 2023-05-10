use resources::{database, user};
mod resources;

mashin_sdk::construct_provider!(
    /// This provider is designed to help you manage resources in a [MySQL](https://www.mysql.com/)
    /// server within your infrastructure, all with the robustness and convenience
    /// of Mashin.
    mysql_provider,
    config = {
        /// This is the connection endpoint.
        /// @example "localhost:3307"
        endpoint: String,
        /// This is the connection username.
        /// @example "root"
        username: String,
        /// This is the connection password.
        /// @example "password"
        password: String,
    },
    resources = [database, user],
    state = |state, config| {
        let endpoint = &config.endpoint;
        let username = &config.username;
        let password = &config.password;
        let url = format!("mysql://{username}:{password}@{endpoint}");

        let mysql_opts = mysql::Opts::try_from(url.as_str()).unwrap();
        let pool = mysql::Pool::new(mysql_opts).unwrap();

        let mut state = state.lock();
        state.put(pool);
    },
    on_drop = |_provider| {
        log!(trace, "my provider dropped");
    }
);
