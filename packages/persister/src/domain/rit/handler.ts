import { Err, Ok } from "ts-results-es";
import type { Knex } from "knex";
import { match } from "ts-pattern";
import { getLogger } from "@logtape/logtape";
import { formatDate, parseISO } from "date-fns";
import { tz } from "@date-fns/tz";

import type { Handler } from "../../types";
import type { DateISOString } from "../../types/infoplus";
import { ChangeType } from "../../types/infoplus";
import type { JourneyEventTable } from "../../types/db";
import { JourneyEventType } from "../../types/db";

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

// converts an input ISO datetime string in UTC
// into the local time (HH:MM)
const convertTime = (isoString: string): string =>
  formatDate(parseISO(isoString), "HH:mm:ss", { in: tz("Europe/Amsterdam") });

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
              convertTime(changedStation.AankomstTijd[0].text);

            update.arrival_time_actual =
              changedStation.AankomstTijd &&
              changedStation.AankomstTijd[1] &&
              convertTime(changedStation.AankomstTijd[1]?.text);

            update.arrival_platform_planned =
              changedStation.TreinAankomstSpoor &&
              formatPlatform(changedStation.TreinAankomstSpoor[0]);

            update.arrival_platform_actual =
              changedStation.TreinAankomstSpoor &&
              changedStation.TreinAankomstSpoor[1] &&
              formatPlatform(changedStation.TreinAankomstSpoor[1]);

            update.departure_time_planned =
              changedStation.VertrekTijd &&
              convertTime(changedStation.VertrekTijd[0].text);

            update.departure_time_actual =
              changedStation.VertrekTijd &&
              changedStation.VertrekTijd[1] &&
              convertTime(changedStation.VertrekTijd[1].text);

            update.departure_platform_planned =
              changedStation.TreinVertrekSpoor &&
              formatPlatform(changedStation.TreinVertrekSpoor[0]);

            update.departure_platform_actual =
              changedStation.TreinVertrekSpoor &&
              changedStation.TreinVertrekSpoor[1] &&
              formatPlatform(changedStation.TreinVertrekSpoor[1]);
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

const formatPlatform = ({
  SpoorNummer,
  SpoorFase,
}: {
  SpoorNummer: string;
  SpoorFase?: string;
}) => (SpoorFase ? `${SpoorNummer}${SpoorFase}` : SpoorNummer);

export const handler: Handler<RitMessage> = async (db, data) => {
  const msg = data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo;

  try {
    const changedElements = getChanges(data);
    await handleStationLevelChanges(
      db,
      msg.RitInfo.TreinNummer,
      msg.RitInfo.TreinDatum,
      changedElements[ChangeLevel.Station],
    );
  } catch (e) {
    if (!(e instanceof Error)) {
      return Err("RIT processing failed");
    }

    return Err(`RIT processing failed: ${e.message}`);
  }

  return Ok(void 0);
};
