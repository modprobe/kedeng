import type { Knex } from "knex";

const requireEnv = (env: string): string => {
  const value = process.env[env];
  if (!value) {
    throw new Error(`Environment variable "${env}" is required`);
  }

  return value;
};

const config: Knex.Config = {
  client: "pg",
  connection: {
    user: requireEnv("DB_USER"),
    password: requireEnv("DB_PASSWORD"),
    database: requireEnv("DB_NAME"),
    host: requireEnv("DB_HOST"),
  } as Knex.PgConnectionConfig,

  migrations: {
    extension: "ts",
    directory: "migrations",
    tableName: "_migrations",
    loadExtensions: process.env.NODE_ENV === "production" ? [".js"] : undefined,
  },
};

export default config;
