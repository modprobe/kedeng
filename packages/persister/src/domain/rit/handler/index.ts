import { Result, Err } from "ts-results-es";
import { getLogger } from "@logtape/logtape";

import type { Handler } from "../../../types";
import { LOGGER_CATEGORY, parseDateString } from "../../../utils";
import { type RitMessage } from "../types";
import { ChangeType } from "../../../types/infoplus";
import { existingServiceAndJourney } from "../utils";

import { handleNewJourney } from "./newJourney";
import { handleStationLevelChanges } from "./changedStations";
import { handleAdditionalStations } from "./additionalStations";

export const enum ChangeLevel {
  Journey,
  JourneySegment,
  Station,
}

const logger = getLogger([...LOGGER_CATEGORY, "rit"]);

export const handler: Handler<RitMessage> = async (db, data) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;
  const journeyLegs = msg.LogischeRit.LogischeRitDeel;

  const processingResults: Result<any, string>[] = [];

  for (const journeyLeg of journeyLegs) {
    try {
      const existing = await existingServiceAndJourney(
        db,
        journeyLeg.LogischeRitDeelNummer,
        parseDateString(msg.TreinDatum),
      );

      if (!existing.service_id || !existing.journey_id) {
        logger.info("Inserting new journey", {
          trainNumber: journeyLeg.LogischeRitDeelNummer,
          runningOn: msg.TreinDatum,
        });

        const newJourney = await handleNewJourney(db, data);
        processingResults.push(newJourney);
        continue;
      }

      // debug: log full messages that contain additional passages
      if (
        journeyLeg.LogischeRitDeelStation.some((s) =>
          s.Wijziging?.some(
            (w) => w.WijzigingType === ChangeType.AdditionalPassage,
          ),
        )
      ) {
        logger.info("existing train with additional passages", {
          fullMsg: JSON.stringify(data),
        });
      }

      const additional = await handleAdditionalStations(
        db,
        existing.journey_id,
        journeyLeg,
      );

      const changedStations = journeyLeg.LogischeRitDeelStation.filter(
        (s) => s.Wijziging,
      );

      const changedStationsResult = await handleStationLevelChanges(
        db,
        journeyLeg.LogischeRitDeelNummer,
        msg.TreinDatum,
        changedStations,
      );

      processingResults.push(Result.all([additional, changedStationsResult]));
    } catch (e) {
      if (!(e instanceof Error)) {
        processingResults.push(Err("RIT processing failed"));
        continue;
      }

      logger.error("error while handling RIT message", {
        err: {
          name: e.name,
          message: e.message,
          stack: e.stack,
          cause: e.cause,
        },
        trainNumber: msg.TreinNummer,
        date: msg.TreinDatum,
      });
      processingResults.push(
        Err(`RIT processing failed: ${e.message} - ${e.stack}`),
      );
    }
  }

  return Result.all(processingResults);
};
