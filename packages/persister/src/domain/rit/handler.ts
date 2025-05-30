import { Err, Ok } from "ts-results-es";
import type { Knex } from "knex";
import { match } from "ts-pattern";
import { getLogger } from "@logtape/logtape";

import type { Handler } from "../../types";
import type { DateISOString } from "../../types/infoplus";
import { ChangeType } from "../../types/infoplus";
import type { JourneyEventTable } from "../../types/db";
import { JourneyEventType } from "../../types/db";
import { extractTimeFromIsoString } from "../../utils";

import type {
  Journey,
  JourneySegment,
  JourneySegmentStation,
  RitMessage,
} from "./types";

const logger = getLogger(["kedeng", "persister", "rit"]);

export const enum ChangeLevel {
  Journey,
  JourneySegment,
  Station,
}

export const getChanges = (
  msg: RitMessage,
): {
  [ChangeLevel.Journey]: Journey[];
  [ChangeLevel.JourneySegment]: JourneySegment[];
  [ChangeLevel.Station]: JourneySegmentStation[];
} => {
  const data =
    msg.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;

  return {
    [ChangeLevel.Journey]: data.LogischeRit.Wijziging ? [data.LogischeRit] : [],
    [ChangeLevel.JourneySegment]: data.LogischeRit.LogischeRitDeel.filter(
      (lrd) => lrd.Wijziging,
    ),
    [ChangeLevel.Station]: data.LogischeRit.LogischeRitDeel.flatMap((lrd) =>
      lrd.LogischeRitDeelStation.filter((s) => s.Wijziging),
    ),
  };
};

const handleStationLevelChanges = async (
  db: Knex,
  trainNumber: string,
  runningOn: DateISOString,
  changedStations: JourneySegmentStation[],
) => {
  for (const changedStation of changedStations) {
    const update: JourneyEventTable = {};

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
       * [ ] 43 extra doorkomst
       * [x] 44 vervallen doorkomst
       */

      match(change)
        .with({ WijzigingType: ChangeType.DepartureDelayed }, () => {
          update.departure_time_actual = extractTimeFromIsoString(
            changedStation.VertrekTijd![1]!.text,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalDelayed }, () => {
          update.arrival_time_actual = extractTimeFromIsoString(
            changedStation.AankomstTijd![1]!.text,
          );
        })
        .with({ WijzigingType: ChangeType.DepartureTimeChanged }, () => {
          update.departure_time_planned = extractTimeFromIsoString(
            changedStation.VertrekTijd![0].text,
          );
        })
        .with({ WijzigingType: ChangeType.ArrivalTimeChanged }, () => {
          update.arrival_time_planned = extractTimeFromIsoString(
            changedStation.AankomstTijd![0].text,
          );
        })
        .with({ WijzigingType: ChangeType.DeparturePlatformChanged }, () => {
          update.departure_platform_actual =
            changedStation.TreinVertrekSpoor![1]!.SpoorNummer;
        })
        .with({ WijzigingType: ChangeType.ArrivalPlatformChanged }, () => {
          update.arrival_platform_actual =
            changedStation.TreinAankomstSpoor![1]!.SpoorNummer;
        })
        .with({ WijzigingType: ChangeType.ArrivalPlatformAllocated }, () => {
          update.arrival_platform_planned =
            changedStation.TreinAankomstSpoor![0].SpoorNummer;
        })
        .with({ WijzigingType: ChangeType.DeparturePlatformAllocated }, () => {
          update.departure_platform_planned =
            changedStation.TreinVertrekSpoor![0].SpoorNummer;
        })
        .with(
          { WijzigingType: ChangeType.ArrivalCancelled },
          { WijzigingType: ChangeType.DepartureCancelled },
          { WijzigingType: ChangeType.PassageCancelled },
          () => {
            update.is_cancelled = true;
          },
        )
        .with(
          { WijzigingType: ChangeType.AdditionalArrival },
          { WijzigingType: ChangeType.AdditionalDeparture },
          () => {
            update.event_type_actual =
              changedStation.AankomstTijd?.[0].text !==
              changedStation.VertrekTijd?.[0].text
                ? JourneyEventType.LongerStop
                : JourneyEventType.ShortStop;

            update.arrival_time_planned =
              changedStation.AankomstTijd &&
              extractTimeFromIsoString(changedStation.AankomstTijd[0].text);

            update.arrival_time_actual =
              changedStation.AankomstTijd &&
              changedStation.AankomstTijd[1] &&
              extractTimeFromIsoString(changedStation.AankomstTijd[1]?.text);

            update.arrival_platform_planned =
              changedStation.TreinAankomstSpoor &&
              changedStation.TreinAankomstSpoor[0].SpoorNummer;

            update.arrival_platform_actual =
              changedStation.TreinAankomstSpoor &&
              changedStation.TreinAankomstSpoor[1] &&
              changedStation.TreinAankomstSpoor[1].SpoorNummer;

            update.departure_time_planned =
              changedStation.VertrekTijd &&
              extractTimeFromIsoString(changedStation.VertrekTijd[0].text);

            update.departure_time_actual =
              changedStation.VertrekTijd &&
              changedStation.VertrekTijd[1] &&
              extractTimeFromIsoString(changedStation.VertrekTijd[1].text);

            update.departure_platform_planned =
              changedStation.TreinVertrekSpoor &&
              changedStation.TreinVertrekSpoor[0].SpoorNummer;

            update.departure_platform_actual =
              changedStation.TreinVertrekSpoor &&
              changedStation.TreinVertrekSpoor[1] &&
              changedStation.TreinVertrekSpoor[1].SpoorNummer;
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

    await db<JourneyEventTable>("journey_event")
      .update(update)
      .whereIn("id", (knex) => {
        knex
          .select("journey_event.id")
          .from("journey_event")
          .innerJoin("journey", "journey_event.journey_id", "journey.id")
          .innerJoin("service", "journey.service_id", "service.id")
          .where({
            "service.train_number": trainNumber,
            "journey.running_on": runningOn,
            "journey_event.station":
              changedStation.Station.StationCode.toLowerCase(),
          });
      });
  }
};

export const handler: Handler<RitMessage> = async (db, data) => {
  const msg = data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo;

  try {
    const trx = await db.transaction();
    const changedElements = getChanges(data);
    await handleStationLevelChanges(
      trx,
      msg.RitInfo.TreinNummer,
      msg.RitInfo.TreinDatum,
      changedElements[ChangeLevel.Station],
    );

    await trx.commit();
  } catch (e) {
    if (!(e instanceof Error)) {
      return Err("RIT processing failed");
    }

    return Err(`RIT processing failed: ${e.message}`);
  }

  return Ok(void 0);
};
