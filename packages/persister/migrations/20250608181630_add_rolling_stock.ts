import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  await knex.schema.createTable("rolling_stock", (table) => {
    table.uuid("journey_id").notNullable();
    table.foreign("journey_id").references("journey.id");

    table.uuid("journey_event_id").notNullable();
    table.foreign("journey_event_id").references("journey_event.id");

    table.mediumint("departure_order").notNullable();
    table.unique(["journey_event_id", "departure_order"]);

    table.text("material_type").nullable();
    table.text("material_subtype").nullable();
    table.text("material_number").nullable().index();
  });
}

export async function down(knex: Knex): Promise<void> {
  await knex.schema.dropTable("rolling_stock");
}
