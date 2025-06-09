import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.index("running_on");
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey", (table) => {
    table.dropIndex("running_on");
  });
}
