import { Result, Err, Ok } from "ts-results-es";

import type { Handler } from "../../../types";
import { parseDateString } from "../../../utils";
import { type RitMessage } from "../types";

import { existingServiceAndJourney, handleNewJourney } from "./newJourney";
import { handleStationLevelChanges } from "./changedStations";

export const enum ChangeLevel {
  Journey,
  JourneySegment,
  Station,
}

export const handler: Handler<RitMessage> = async (db, data) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;
  const journeyLegs = msg.LogischeRit.LogischeRitDeel;

  const processingResults: Result<void, string>[] = [];

  for (const journeyLeg of journeyLegs) {
    try {
      const existing = await existingServiceAndJourney(
        db,
        journeyLeg.LogischeRitDeelNummer,
        parseDateString(msg.TreinDatum),
      );

      if (!existing.service_id || !existing.journey_id) {
        await handleNewJourney(db, data);
        processingResults.push(Ok(undefined));
        continue;
      }

      const changedStations = journeyLeg.LogischeRitDeelStation.filter(
        (s) => s.Wijziging,
      );

      await handleStationLevelChanges(
        db,
        journeyLeg.LogischeRitDeelNummer,
        msg.TreinDatum,
        changedStations,
      );

      processingResults.push(Ok(void 0));
    } catch (e) {
      if (!(e instanceof Error)) {
        processingResults.push(Err("RIT processing failed"));
        continue;
      }

      processingResults.push(Err(`RIT processing failed: ${e.message}`));
    }
  }

  return Result.all(processingResults);
};
