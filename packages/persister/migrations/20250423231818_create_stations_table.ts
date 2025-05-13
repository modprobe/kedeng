import type { Knex } from "knex";

export async function up(knex: Knex): Promise<void> {
  return knex.schema.createTable("station", (table) => {
    table.text("uic_code").primary().notNullable();
    table.text("uic_cd_code").unique();
    table.text("eva_code").unique();
    table.integer("cd_code").unique();
    table.text("code").notNullable().unique();

    table
      .enum("station_type", [
        "EXPRESS_TRAIN_STATION",
        "INTERCITY_HUB_STATION",
        "INTERCITY_STATION",
        "LOCAL_TRAIN_STATION",
        "EXPRESS_TRAIN_HUB_STATION",
        "MEGA_STATION",
        "LOCAL_TRAIN_HUB_STATION",
        "OPTIONAL_STATION",
      ])
      .notNullable();

    table.text("name_long").notNullable();
    table.text("name_medium");
    table.text("name_short");
    table.specificType("name_synonyms", "text[]");

    table.string("country").notNullable().index();
    table.specificType("tracks", "text[]");

    table.boolean("has_travel_assistance");
    table.boolean("is_border_stop");
    table.boolean("is_available_for_accessible_travel");
    table.boolean("has_known_facilities");
    table.boolean("are_tracks_independently_accessible");

    table.point("location");
  });
}

export async function down(knex: Knex): Promise<void> {
  return knex.schema.dropTable("station");
}
