import type { Knex } from "knex";
import { match } from "ts-pattern";
import { getLogger } from "@logtape/logtape";
import { Err, Ok, Result } from "ts-results-es";
import type { JourneyEvent } from "knex/types/tables";

import { type DateISOString, ChangeType } from "../../../types/infoplus";
import type { JourneySegmentStation } from "../types";
import { convertTime, formatPlatform } from "../utils";
import { LOGGER_CATEGORY } from "../../../utils";

const logger = getLogger([...LOGGER_CATEGORY, "rit", "changedStations"]);

export const handleStationLevelChanges = async (
  db: Knex,
  trainNumber: string,
  runningOn: DateISOString,
  changedStations: JourneySegmentStation[],
): Promise<Result<any, string>> => {
  const results: Result<any, string>[] = [];

  for (const changedStation of changedStations) {
    const update: Partial<JourneyEvent> = {};

    for (const change of changedStation.Wijziging ?? []) {
      /**
       * Possible changes on this level:
       * [x] 10 vertrekvertraging
       * [x] 11 aankomstvertraging
       * [x] 12 vertrektijd gewijzigd
       * [x] 13 aankomsttijd gewijzigd
       * [x] 20 vertrekspoor gew
       * [x] 21 aankomstspoor gew
       * [x] 22 vertrekspoorfixatie
       * [x] 23 aankomstspoorfixatie
       * [x] 31 extra vertrek
       * [x] 32 vervallen vertrek
       * [x] 38 extra aankomst
       * [x] 39 vervallen aankomst
       * [x] 43 extra doorkomst
       * [x] 44 vervallen doorkomst
       */
      match(change)
        .with({ WijzigingType: ChangeType.DepartureDelayed }, () => {
          update.departure_time_actual = convertTime(
            changedStation.VertrekTijd![1]!.text,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalDelayed }, () => {
          update.arrival_time_actual = convertTime(
            changedStation.AankomstTijd![1]!.text,
          );
        })
        .with({ WijzigingType: ChangeType.DepartureTimeChanged }, () => {
          update.departure_time_planned = convertTime(
            changedStation.VertrekTijd![0].text,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalTimeChanged }, () => {
          update.arrival_time_planned = convertTime(
            changedStation.AankomstTijd![0].text,
          );
        })
        .with({ WijzigingType: ChangeType.DeparturePlatformChanged }, () => {
          update.departure_platform_actual = formatPlatform(
            changedStation.TreinVertrekSpoor![1]!,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalPlatformChanged }, () => {
          update.arrival_platform_actual = formatPlatform(
            changedStation.TreinAankomstSpoor![1]!,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalPlatformAllocated }, () => {
          update.arrival_platform_planned = formatPlatform(
            changedStation.TreinAankomstSpoor![0],
          );
        })
        .with({ WijzigingType: ChangeType.DeparturePlatformAllocated }, () => {
          update.departure_platform_planned = formatPlatform(
            changedStation.TreinVertrekSpoor![0],
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalCancelled }, () => {
          update.arrival_cancelled = true;
        })
        .with({ WijzigingType: ChangeType.DepartureCancelled }, () => {
          update.departure_cancelled = true;
        })
        .with({ WijzigingType: ChangeType.PassageCancelled }, () => {
          update.arrival_cancelled = true;
          update.departure_cancelled = true;
        })
        .with(
          { WijzigingType: ChangeType.AdditionalArrival },
          { WijzigingType: ChangeType.AdditionalDeparture },
          { WijzigingType: ChangeType.AdditionalPassage },
          () => {
            // noop, handled separately
          },
        )
        .otherwise((change) => {
          logger.info(`Unhandled change of type "${change.WijzigingType}"`, {
            change,
            changedStation,
            trainNumber,
            runningOn,
          });
        });
    }

    if (Object.keys(update).length === 0) {
      results.push(Err("no updates"));
      continue;
    }

    await db("journey_event")
      .update(update)
      .whereIn("id", (knex) => {
        knex
          .select("journey_event.id")
          .from("journey_event")
          .innerJoin("journey", "journey_event.journey_id", "journey.id")
          .innerJoin("service", "journey.service_id", "service.id")
          .where("service.train_number", trainNumber)
          .andWhere("journey.running_on", runningOn)
          .andWhere(
            "journey_event.station",
            changedStation.Station.StationCode.toLowerCase(),
          );
      });

    results.push(Ok(undefined));
  }

  return Result.all(results);
};
