import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("service", (table) => {
    table.unique(["timetable_year", "train_number"]);
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("service", (table) => {
    table.dropUnique(["timetable_year", "train_number"]);
  });
}
