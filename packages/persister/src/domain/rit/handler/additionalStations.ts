import type { Knex } from "knex";
import { Err, Ok, type Result } from "ts-results-es";
import type { JourneyEvent } from "knex/types/tables";
import { getLogger } from "@logtape/logtape";

import { ChangeType } from "../../../types/infoplus";
import type { JourneySegment } from "../types";
import { determineJourneyEventType, formatPlatform } from "../utils";
import { LOGGER_CATEGORY } from "../../../utils";

const logger = getLogger([...LOGGER_CATEGORY, "rit", "additional"]);

export const handleAdditionalStations = async (
  db: Knex,
  journeyId: string,
  journey: JourneySegment,
): Promise<Result<any, string>> => {
  // return early if no additional stops/passages are indicated
  if (
    !journey.LogischeRitDeelStation.some(
      (station) =>
        station.Wijziging?.some((change) =>
          [
            ChangeType.AdditionalArrival,
            ChangeType.AdditionalDeparture,
            ChangeType.AdditionalPassage,
          ].includes(change.WijzigingType),
        ) ?? false,
    )
  ) {
    return Ok(undefined);
  }

  const inserts: Knex.DbRecordArr<JourneyEvent> =
    journey.LogischeRitDeelStation.map((station, idx) => ({
      journey_id: journeyId,

      station: station.Station.StationCode.toLowerCase(),
      stop_order: idx,

      event_type_planned: determineJourneyEventType(station),
      event_type_actual: determineJourneyEventType(station),

      arrival_time_planned: station.AankomstTijd?.[0]?.text,
      arrival_time_actual: station.AankomstTijd?.[1]?.text,
      arrival_platform_planned:
        station.TreinAankomstSpoor?.[0] &&
        formatPlatform(station.TreinAankomstSpoor[0]),
      arrival_platform_actual:
        station.TreinAankomstSpoor?.[1] &&
        formatPlatform(station.TreinAankomstSpoor[1]),

      departure_time_planned: station.VertrekTijd?.[0]?.text,
      departure_time_actual: station.VertrekTijd?.[1]?.text,
      departure_platform_planned:
        station.TreinVertrekSpoor?.[0] &&
        formatPlatform(station.TreinVertrekSpoor[0]),
      departure_platform_actual:
        station.TreinVertrekSpoor?.[1] &&
        formatPlatform(station.TreinVertrekSpoor[1]),
    }));

  try {
    await db("journey_event")
      .insert(inserts)
      .onConflict(["journey_id", "stop_order"])
      .merge();
  } catch (e) {
    logger.error("failed to insert updated/additional journey events", {
      trainNumber: journey.LogischeRitDeelNummer,
      journeyId,
      ...(e instanceof Error
        ? {
            err: {
              name: e.name,
              message: e.message,
              stack: e.stack,
              cause: e.cause,
            },
          }
        : {}),
    });

    return Err("failed to insert updated journey events");
  }

  return Ok(undefined);
};
