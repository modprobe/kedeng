import assert from "node:assert";

import type { Knex } from "knex";
import type { JourneyEvent } from "knex/types/tables";
import { Ok } from "ts-results-es";

import type { JourneySegment, RitMessage } from "../types";

import { logger, ritJourneyToDbJourneyEvents, RitResult } from ".";

export const insertNewJourney = async (
  db: Knex,
  journey: JourneySegment,
  data: RitMessage,
  serviceId?: string,
) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;
  const provider = msg.Vervoerder;
  const trainType = msg.TreinSoort.attr.Code;
  const runningOn = msg.TreinDatum;

  if (!serviceId) {
    const insertedServiceIds = await db("service")
      .insert({
        train_number: journey.LogischeRitDeelNummer,
        provider,
        type: trainType,
      })
      .returning("id");

    serviceId = insertedServiceIds[0].id;

    logger.debug("Inserted new service", {
      trainNumber: journey.LogischeRitDeelNummer,
      runningOn,
      serviceId,
    });
  }

  const insertedJourneyIds = await db("journey")
    .insert({
      service_id: serviceId,
      running_on: runningOn,
    })
    .returning("id");

  const journeyId = insertedJourneyIds[0].id;
  logger.debug("Inserted new journey", {
    trainNumber: journey.LogischeRitDeelNummer,
    runningOn,
    journeyId,
  });

  const journeyEventsToInsert: Knex.DbRecordArr<JourneyEvent> =
    ritJourneyToDbJourneyEvents(journey, journeyId);

  assert.deepStrictEqual(
    journeyEventsToInsert.map((evt) => evt.stop_order),
    [...new Array(journeyEventsToInsert.length).keys()],
    "new journey: stop_order not sequential as expected",
  );

  await db("journey_event").insert(journeyEventsToInsert);
  return Ok(RitResult.INSERTED_NEW);
};
