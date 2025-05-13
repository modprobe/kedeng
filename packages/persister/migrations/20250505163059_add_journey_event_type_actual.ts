import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.renameColumn("event_type", "event_type_planned");
    table
      .enum("event_type_actual", [
        "ARRIVAL",
        "DEPARTURE",
        "SHORT_STOP",
        "LONGER_STOP",
        "PASSAGE",
      ])
      .nullable();

    table.boolean("is_cancelled").nullable();
  });
}

export async function down(knex: Knex): Promise<void> {
  return knex.schema.alterTable("journey_event", (table) => {
    table.dropColumn("event_type_actual");
    table.renameColumn("event_type_planned", "event_type");
    table.dropColumn("is_cancelled");
  });
}
