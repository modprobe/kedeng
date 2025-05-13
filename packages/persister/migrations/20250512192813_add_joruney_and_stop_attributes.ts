import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.specificType("attributes", "text[]");
  });

  await knex.schema.alterTable("journey_event", (table) => {
    table.specificType("attributes", "text[]");
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.dropColumn("attributes");
  });

  await knex.schema.alterTable("journey_event", (table) => {
    table.dropColumn("attributes");
  });
}
