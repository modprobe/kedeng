import type { Knex } from "knex";
import type { JourneyEvent } from "knex/types/tables";
import { match, P } from "ts-pattern";
import { Ok } from "ts-results-es";
import { format as formatDate } from "date-fns";

import type { Handler } from "../../../types";
import { JourneyEventType } from "../../../types/db";
import { parseDateString } from "../../../utils";
import { type RitMessage, StopType } from "../types";
import { convertTime, formatPlatform } from "../utils";

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
        const eventType: JourneyEventType = match([
          stop.StationnementType,
          stop.Stopt[1]?.text,
          stop.AankomstTijd?.[0]?.text,
          stop.VertrekTijd?.[0]?.text,
        ])
          .returnType<JourneyEventType>()
          .with(
            [StopType.Passage, "N", P._, P._],
            () => JourneyEventType.Passage,
          )
          .with(
            [StopType.Stop, "J", undefined, P.string],
            () => JourneyEventType.Departure,
          )
          .with(
            [StopType.Stop, "J", P.string, undefined],
            () => JourneyEventType.Arrival,
          )
          .with(
            [StopType.Stop, "J", P.string, P.string],
            ([_a, _b, arrival, departure]) =>
              arrival === departure
                ? JourneyEventType.ShortStop
                : JourneyEventType.LongerStop,
          )
          .run();

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
        };
      });

    await db("journey_event").insert(journeyEventValues);
  }

  return Ok(undefined);
};

export const existingServiceAndJourney = async (
  db: Knex,
  trainNumber: string,
  date: Date,
): Promise<{ service_id: string | null; journey_id: string | null }> => {
  const existing = (await db
    .table<{ id: string; train_number: string }>("service")
    .leftJoin("journey", (on) => {
      on.on("service.id", "journey.service_id").andOn(
        "journey.running_on",
        db.raw("?", formatDate(date, "yyyy-MM-dd")),
      );
    })
    .select({ service_id: "service.id", journey_id: "journey.id" })
    .where("service.train_number", trainNumber)
    .andWhere("service.timetable_year", formatDate(date, "yyyy"))
    .limit(1)) as [
    | {
        service_id: string;
        journey_id: string | null;
      }
    | undefined,
  ];

  return existing[0] ?? { service_id: null, journey_id: null };
};
