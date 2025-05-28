import { getLogger } from "@logtape/logtape";
import knex from "knex";

export const createDbConnection = async () => {
  const { default: config } = await import("../knexfile.js");

  const logger = getLogger(["kedeng", "persister", "knex"]);

  // @ts-expect-error whatever
  const conn = knex(config);
  conn.on("query", (data) => logger.debug("Query logged", { data }));

  return conn;
};
