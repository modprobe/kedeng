import assert from "node:assert";

import type { Knex } from "knex";
import type { JourneyEvent, RollingStock } from "knex/types/tables";
import { Ok } from "ts-results-es";
import uuid from "uuid";

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
        id: uuid.v7(),
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
      id: uuid.v7(),
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

  const journeyEventsToInsert = ritJourneyToDbJourneyEvents(journey, journeyId);

  assert.deepStrictEqual(
    journeyEventsToInsert.map((evt) => evt.stop_order),
    [...new Array(journeyEventsToInsert.length).keys()],
    "new journey: stop_order not sequential as expected",
  );

  await db("journey_event").insert(journeyEventsToInsert);
  await db("rolling_stock").insert(
    extractRollingStockEntries(journey, journeyEventsToInsert),
  );

  return Ok(RitResult.INSERTED_NEW);
};

const extractRollingStockEntries = (
  journey: JourneySegment,
  journeyEvents: Partial<JourneyEvent>[],
): RollingStock[] => {
  const rollingStockEntries = [];

  for (const journeyEvent of journeyEvents) {
    const rollingStock = journey.LogischeRitDeelStation.find(
      (stop) =>
        stop.Station?.StationCode.toLowerCase() ===
        journeyEvent.station?.toLowerCase(),
    )?.MaterieelDeel;

    const preparedRollingStockEntries: RollingStock[] =
      rollingStock
        ?.filter((rs) => rs.AchterBlijvenMaterieelDeel === "N")
        ?.map((rs) => ({
          journey_id: journeyEvent.journey_id!,
          journey_event_id: journeyEvent.id!,
          departure_order: parseInt(rs.MaterieelDeelVolgordeVertrek, 10),

          material_type: rs.MaterieelDeelSoort,
          material_subtype: rs.MaterieelDeelAanduiding,
          material_number: rs.MaterieelDeelID,
        })) ?? [];

    rollingStockEntries.push(...preparedRollingStockEntries);
  }

  return rollingStockEntries;
};
