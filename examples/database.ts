#!/usr/bin/env mashin run
import * as mysql from "../mod.ts";

const provider = new mysql.Provider("mysql", {
  endpoint: "localhost:3306",
  username: "root",
  password: "pw",
});

const database = new mysql.Database(
  "database_name",
  {
    defaultCharacterSet: "utf8mb4",
  },
  { provider }
);
