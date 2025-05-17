import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  return knex.schema

    .createTable("service", (table) => {
      table
        .uuid("id")
        .primary()
        .defaultTo(knex.raw("gen_random_uuid()"))
        .notNullable();
      table.text("train_number").notNullable();
      table
        .text("timetable_year")
        .notNullable()
        .defaultTo(knex.raw("date_part('year'::text, now())"));

      table.text("type").notNullable();
      table.text("provider").notNullable();

      table.index(["type", "train_number"]);
    })

    .createTable("journey", (table) => {
      table
        .uuid("id")
        .primary()
        .defaultTo(knex.raw("gen_random_uuid()"))
        .notNullable();

      table.uuid("service_id").notNullable();
      table.date("running_on").notNullable();

      table
        .foreign("service_id")
        .references("service.id")
        .deferrable("deferred");

      table.unique(["service_id", "running_on"]);
    })

    .createTable("journey_event", (table) => {
      table.uuid("id").primary().defaultTo(knex.raw("gen_random_uuid()"));
      table.uuid("journey_id").notNullable().index();

      table
        .foreign("journey_id")
        .references("journey.id")
        .deferrable("deferred");

      table.text("station").notNullable();
      table
        .enum("event_type", [
          "ARRIVAL",
          "DEPARTURE",
          "SHORT_STOP",
          "LONGER_STOP",
          "PASSAGE",
        ])
        .defaultTo("SHORT_STOP")
        .notNullable();

      table.integer("stop_order").notNullable();

      table.time("arrival_time_planned");
      table.time("arrival_time_actual");
      table.text("arrival_platform_planned");
      table.text("arrival_platform_actual");

      table.time("departure_time_planned");
      table.time("departure_time_actual");
      table.text("departure_platform_planned");
      table.text("departure_platform_actual");
    });
}

export async function down(knex: Knex): Promise<void> {
  return knex.schema
    .dropTable("journey_event")
    .dropTable("journey")
    .dropTable("service");
}
