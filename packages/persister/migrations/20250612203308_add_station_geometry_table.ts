import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.createTable("station_geometry", (table) => {
    table.text("from").notNullable();
    table.text("to").notNullable();

    table.jsonb("line_string").notNullable();

    table.unique(["from", "to"]);
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.dropTable("station_geometry");
}
