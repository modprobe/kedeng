import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("rolling_stock", (table) => {
    table.dropForeign("journey_id");
    table.foreign("journey_id").references("journey.id").onDelete("CASCADE");

    table.dropForeign("journey_event_id");
    table
      .foreign("journey_event_id")
      .references("journey_event.id")
      .onDelete("CASCADE");
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.alterTable("rolling_stock", (table) => {
    table.dropForeign("journey_id");
    table.foreign("journey_id").references("journey.id");

    table.dropForeign("journey_event_id");
    table.foreign("journey_event_id").references("journey_event.id");
  });
}
