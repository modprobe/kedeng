import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.specificType("source_ids", "text[]").nullable();
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.dropColumn("source_ids");
  });
}
