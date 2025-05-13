import type { Knex } from "knex";
import type { Result } from "ts-results-es";

export type Handler<TData> = (
  db: Knex,
  data: TData,
) => Promise<Result<void, string>>;
