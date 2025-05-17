import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.unique(["journey_id", "stop_order"]);
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.dropUnique(["journey_id", "stop_order"]);
  });
}
