import type { Knex } from "knex";
import type { JourneyEvent } from "knex/types/tables";
import { match } from "ts-pattern";
import { Ok } from "ts-results-es";

import type { Handler } from "../../../types";
import type { JourneyEventType } from "../../../types/db";
import { parseDateString } from "../../../utils";
import { type RitMessage, StopType } from "../types";
import {
  convertTime,
  determineJourneyEventType,
  existingServiceAndJourney,
  formatPlatform,
} from "../utils";
import { Attributes } from "../../../types/infoplus";

export const handleNewJourney: Handler<RitMessage> = async (db, data) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductRitInfo.RitInfo;

  const runningDate = parseDateString(msg.TreinDatum);

  const journeys = msg.LogischeRit.LogischeRitDeel;

  for (const journey of journeys) {
    const existing = await existingServiceAndJourney(
      db,
      journey.LogischeRitDeelNummer,
      runningDate,
    );

    let serviceId = existing.service_id;
    if (serviceId === null) {
      const inserted = await db("service")
        .insert({
          train_number: journey.LogischeRitDeelNummer,
          type: msg.TreinSoort.attr.Code,
          provider: msg.Vervoerder,
        })
        .returning("id");

      serviceId = inserted[0].id;
    }

    const insertedJourney = await db("journey")
      .insert({
        service_id: serviceId,
        running_on: msg.TreinDatum,
      })
      .returning("id");

    const journeyId = insertedJourney[0].id;

    const journeyEventValues: Knex.DbRecordArr<JourneyEvent> =
      journey.LogischeRitDeelStation.map((stop, idx) => {
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

          event_type_planned: eventType,
          event_type_actual: eventType,

          stop_order: idx,

          arrival_time_planned:
            stop.AankomstTijd?.[0].text &&
            convertTime(stop.AankomstTijd[0].text),

          arrival_time_actual:
            stop.AankomstTijd?.[1]?.text &&
            convertTime(stop.AankomstTijd[1].text),

          arrival_platform_planned:
            stop.TreinAankomstSpoor?.[0] &&
            formatPlatform(stop.TreinAankomstSpoor[0]),

          arrival_platform_actual:
            stop.TreinAankomstSpoor?.[1] &&
            formatPlatform(stop.TreinAankomstSpoor[1]),

          departure_time_planned:
            stop.VertrekTijd?.[0].text && convertTime(stop.VertrekTijd[0].text),

          departure_time_actual:
            stop.VertrekTijd?.[1]?.text &&
            convertTime(stop.VertrekTijd[1].text),

          departure_platform_planned:
            stop.TreinVertrekSpoor?.[0] &&
            formatPlatform(stop.TreinVertrekSpoor[0]),

          departure_platform_actual:
            stop.TreinVertrekSpoor?.[1] &&
            formatPlatform(stop.TreinVertrekSpoor[1]),

          attributes,
        };
      });

    await db("journey_event").insert(journeyEventValues);
  }

  return Ok(undefined);
};
