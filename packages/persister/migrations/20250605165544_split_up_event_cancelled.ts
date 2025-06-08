import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.renameColumn("is_cancelled", "departure_cancelled");
    table.boolean("arrival_cancelled");
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.dropColumn("arrival_cancelled");
    table.renameColumn("departure_cancelled", "is_cancelled");
  });
}
