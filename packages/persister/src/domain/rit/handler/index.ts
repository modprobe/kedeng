import assert from "assert";
import { randomUUID } from "crypto";

import type { Knex } from "knex";
import type { JourneyEvent, RollingStock } from "knex/types/tables";
import { match } from "ts-pattern";
import { Ok, Result } from "ts-results-es";
import { getLogger } from "@logtape/logtape";

import type { Handler } from "../../../types";
import { LOGGER_CATEGORY, parseDateString } from "../../../utils";
import type { JourneyEventType } from "../../../types/db";
import { Attributes } from "../../../types/infoplus";
import type {
  JourneySegmentStation,
  JourneySegment,
  RitMessage,
} from "../types";
import { StopType } from "../types";
import { convertTime, formatPlatform } from "../utils";

import {
  determineJourneyEventType,
  existingServiceAndJourney,
  inlineSplittingJourneys,
} from "./utils";
import { insertNewJourney } from "./newJourney";

export const logger = getLogger([...LOGGER_CATEGORY, "rit"]);

export const enum RitResult {
  INSERTED_NEW = "Successfully inserted new journey",
  UPDATED_EXISTING = "Successfully updated existing journey",
}

const ritJourneyStopToJourneyEvent = (
  stop: JourneySegmentStation,
  order: number,
  journeyId?: string,
) => {
  const eventType: JourneyEventType = determineJourneyEventType(stop);

  const attributes: Attributes[] = match(stop.StationnementType)
    .with(StopType.AlightingOnly, () => [Attributes.DoNotBoard])
    .with(StopType.BoardingOnly, () => [Attributes.DoNotAlight])
    .with(StopType.ServiceStop, () => [
      Attributes.DoNotBoard,
      Attributes.DoNotAlight,
    ])
    .otherwise(() => []);

  return {
    journey_id: journeyId,
    station: stop.Station.StationCode.toLowerCase(),

    event_type_actual: eventType,

    stop_order: order,

    arrival_time_planned:
      stop.AankomstTijd?.[0]?.text && convertTime(stop.AankomstTijd[0].text),

    arrival_time_actual:
      stop.AankomstTijd?.[1]?.text && convertTime(stop.AankomstTijd[1].text),

    arrival_platform_planned:
      stop.TreinAankomstSpoor?.[0] &&
      formatPlatform(stop.TreinAankomstSpoor[0]),

    arrival_platform_actual:
      stop.TreinAankomstSpoor?.[1] &&
      formatPlatform(stop.TreinAankomstSpoor[1]),

    departure_time_planned:
      stop.VertrekTijd?.[0]?.text && convertTime(stop.VertrekTijd[0].text),

    departure_time_actual:
      stop.VertrekTijd?.[1]?.text && convertTime(stop.VertrekTijd[1].text),

    departure_platform_planned:
      stop.TreinVertrekSpoor?.[0] && formatPlatform(stop.TreinVertrekSpoor[0]),

    departure_platform_actual:
      stop.TreinVertrekSpoor?.[1] && formatPlatform(stop.TreinVertrekSpoor[1]),

    attributes,
  };
};

export const ritJourneyToDbJourneyEvents = (
  journey: JourneySegment,
  journeyId: string,
) =>
  journey.LogischeRitDeelStation.map((stop, idx) =>
    ritJourneyStopToJourneyEvent(stop, idx, journeyId),
  );

export const handler: Handler<RitMessage> = async (
  db,
  data,
): Promise<Result<string[], string>> => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;
  const runningOn = parseDateString(msg.TreinDatum);

  const results: Result<string, string>[] = [];

  const journeyLegs = inlineSplittingJourneys(msg.LogischeRit);

  for (const journey of journeyLegs) {
    const { journey_id: journeyId, service_id: serviceId } =
      await existingServiceAndJourney(
        db,
        journey.LogischeRitDeelNummer,
        runningOn,
      );

    if (!serviceId || !journeyId) {
      const insertNewResult = await insertNewJourney(
        db,
        journey,
        data,
        serviceId ?? undefined,
      );
      results.push(insertNewResult);
      continue;
    }

    const existingJourneyEvents = await db("journey_event")
      .select()
      .where({
        journey_id: journeyId,
      })
      .orderBy("stop_order", "asc");

    const desiredJourneyEvents = ritJourneyToDbJourneyEvents(
      journey,
      journeyId,
    );

    const allStations = desiredJourneyEvents.map((evt) => evt.station);
    assert.strictEqual(
      allStations.length,
      new Set(allStations).size,
      "Train stops at or passes through one station more than once",
    );

    const resultingJourneyEvents = [];
    const resultingRollingStockEntries = [];

    for (const [order, desiredJourneyEvent] of desiredJourneyEvents.entries()) {
      const matching = existingJourneyEvents.find(
        (event) => event.station === desiredJourneyEvent.station,
      );

      const newEventId = randomUUID();

      const rollingStock = journey.LogischeRitDeelStation.find(
        (stop) =>
          stop.Station.StationCode.toLowerCase() ===
          desiredJourneyEvent.station,
      )?.MaterieelDeel;

      const preparedRollingStockEntries: Knex.DbRecordArr<RollingStock> =
        rollingStock
          ?.filter((stock) => stock.AchterBlijvenMaterieelDeel === "N")
          ?.map((stock) => ({
            journey_event_id: newEventId,
            journey_id: journeyId,
            departure_order: parseInt(stock.MaterieelDeelVolgordeVertrek ?? 1),

            material_type: stock.MaterieelDeelSoort,
            material_subtype: stock.MaterieelDeelAanduiding,
            material_number: stock.MaterieelDeelID,
          })) ?? [];

      const mergedAttributes = [
        ...new Set([
          ...(matching?.attributes ?? []),
          ...(desiredJourneyEvent.attributes ?? []),
        ]),
      ];

      const mergedJourneyEvent: Partial<JourneyEvent> = {
        ...(matching ? matching : {}),
        ...desiredJourneyEvent,
        id: newEventId,
        stop_order: order,
        attributes: mergedAttributes.length > 0 ? mergedAttributes : null,
      };

      resultingJourneyEvents.push(mergedJourneyEvent);
      resultingRollingStockEntries.push(...preparedRollingStockEntries);
    }

    const existingJourneyEventIds = existingJourneyEvents.map(
      (event) => event.id,
    );

    await db("rolling_stock")
      .delete()
      .where({
        journey_id: journeyId,
      })
      .orWhereIn("journey_event_id", existingJourneyEventIds);

    logger.debug("deleted existing rolling stock entries", {
      journeyId,
    });

    await db("journey_event")
      .delete()
      .where({
        journey_id: journeyId,
      })
      .orWhereIn("id", existingJourneyEventIds);

    logger.debug("deleted existing journey events", {
      journeyId,
    });

    await db("journey_event").insert(resultingJourneyEvents);
    logger.debug("inserted updated journey events", {
      journeyId,
    });

    if (resultingRollingStockEntries.length > 0) {
      await db("rolling_stock").insert(resultingRollingStockEntries);
      logger.debug("inserted updated rolling stock entries", {
        journeyId,
      });
    }

    results.push(Ok(RitResult.UPDATED_EXISTING));
  }

  return Result.all(results);
};
