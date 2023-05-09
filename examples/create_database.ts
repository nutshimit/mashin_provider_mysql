#!/usr/bin/env mashin run
import * as mysql from "../mod.ts";

// configure example provider
const mysqlProvider = new mysql.Provider("test_provider", {
  endpoint: "localhost:3306",
  password: "pw",
  username: "root",
});

const new_db = new mysql.Database(
  "mynewdatabase",
  {
    defaultCharacterSet: "utf8mb4",
  },
  { provider: mysqlProvider }
);

const new_user = new mysql.User(
  "david2",
  {
    host: "172.17.0.1",
    plaintextPassword: "pw",
    tlsOption: "NONE",
  },
  { provider: mysqlProvider }
);
