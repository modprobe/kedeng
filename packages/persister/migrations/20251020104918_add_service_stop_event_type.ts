import type { Knex } from "knex";

const newAllowedValues = [
  "ARRIVAL",
  "DEPARTURE",
  "SHORT_STOP",
  "LONGER_STOP",
  "PASSAGE",
  "SERVICE_STOP",
];

export async function up(knex: Knex): Promise<void> {
  await knex.schema.alterTable("journey_event", (table) => {
    table.dropChecks([
      "journey_event_event_type_check",
      "journey_event_event_type_actual_check",
    ]);

    table.text("event_type_planned").checkIn(newAllowedValues).alter();
    table.text("event_type_actual").checkIn(newAllowedValues).alter();
  });
}

export async function down(knex: Knex): Promise<void> {
  const previousValues = newAllowedValues.slice(0, -1);
  await knex.schema.alterTable("journey_event", (table) => {
    table.dropChecks([
      "journey_event_event_type_check",
      "journey_event_event_type_actual_check",
    ]);

    table.text("event_type_planned").checkIn(previousValues).alter();
    table.text("event_type_actual").checkIn(previousValues).alter();
  });
}
