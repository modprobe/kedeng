import type { Knex } from "knex";
import { Ok } from "ts-results-es";
import { format, parseISO } from "date-fns";
import { utc } from "@date-fns/utc";
import { tz } from "@date-fns/tz";

import type { Handler } from "../../types/handler";

import type { DvsMessage } from "./types";

export const handle: Handler<DvsMessage> = async (
  db: Knex,
  data: DvsMessage,
) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductDVS
      .DynamischeVertrekStaat;

  const actualDepartureDatetime =
    msg.Trein.VertrekTijd[1]?.text &&
    parseISO(msg.Trein.VertrekTijd[1].text, {
      in: utc,
    });
  const departureStatus = parseInt(msg.Trein.TreinStatus);
  const actualDeparturePlatform = msg.Trein.TreinVertrekSpoor?.[1]?.SpoorNummer;

  await db("journey_event")
    .update({
      status: db.raw('GREATEST("journey_event"."status", ?)', departureStatus),
      ...(actualDepartureDatetime
        ? {
            departure_time_actual: format(actualDepartureDatetime, "HH:mm:ss", {
              in: tz("Europe/Amsterdam"),
            }),
          }
        : {}),
      ...(actualDeparturePlatform
        ? { departure_platform_actual: actualDeparturePlatform }
        : {}),
    })
    .whereIn("id", (knex) => {
      knex
        .select("journey_event.id")
        .from("journey_event")
        .innerJoin("journey", "journey_event.journey_id", "journey.id")
        .innerJoin("service", "journey.service_id", "service.id")
        .where({
          "service.train_number": msg.RitId,
          "journey.running_on": msg.RitDatum,
          "journey_event.station": msg.RitStation.StationCode.toLowerCase(),
        });
    });

  return Ok(void 0);
};
