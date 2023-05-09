#!/usr/bin/env mashin run
import * as mysql from "../mod.ts";

const provider = new mysql.Provider("mysql", {
  endpoint: "localhost:3306",
  username: "root",
  password: "pw",
});

const user = new mysql.User(
  "david2",
  {
    host: "172.17.0.1",
    plaintextPassword: "pw",
    tlsOption: "NONE",
  },
  { provider }
);
