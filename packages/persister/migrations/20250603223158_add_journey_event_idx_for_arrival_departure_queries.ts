import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.index(["station", "event_type_planned", "event_type_actual"]);
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.dropIndex(["station", "event_type_planned", "event_type_actual"]);
  });
}
