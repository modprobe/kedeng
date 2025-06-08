import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.text("event_type_planned").nullable().alter({ alterType: false });
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table
      .text("event_type_planned")
      .notNullable()
      .defaultTo("SHORT_STOP")
      .alter({ alterType: false });
  });
}
