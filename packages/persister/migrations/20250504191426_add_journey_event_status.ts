import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  return knex.schema.alterTable("journey_event", (table) =>
    table.smallint("status").nullable().defaultTo(null),
  );
}

export async function down(knex: Knex): Promise<void> {
  return knex.schema.alterTable("journey_event", (table) =>
    table.dropColumn("status"),
  );
}
