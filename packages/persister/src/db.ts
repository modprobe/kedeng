import { getLogger } from "@logtape/logtape";
import knex from "knex";

export const createDbConnection = async () => {
  const { default: config } = await import("../knexfile");

  const logger = getLogger(["kedeng", "persister", "knex"]);

  const conn = knex(config);
  conn.on("query", (data) => logger.debug("Query logged", { data }));

  return conn;
};
