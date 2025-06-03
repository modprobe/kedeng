import { Ok } from "ts-results-es";
// import { getLogger } from "@logtape/logtape";
import { format, parseISO } from "date-fns";
import { utc } from "@date-fns/utc";
import { tz } from "@date-fns/tz";

import type { Handler } from "../../types/handler";

import type { DasMessage } from "./types";

// const logger = getLogger(["kedeng", "persister", "DAS"]);

export const handle: Handler<DasMessage> = async (db, data) => {
  const msg =
    data.PutReisInformatieBoodschapIn.ReisInformatieProductDAS
      .DynamischeAankomstStaat;

  const arrivalStatus = msg.TreinAankomst.TreinStatus;
  const actualArrivalDatetime =
    msg.TreinAankomst.AankomstTijd[1]?.text &&
    parseISO(msg.TreinAankomst.AankomstTijd[1]?.text, { in: utc });

  const actualArrivalPlatform =
    msg.TreinAankomst.TreinAankomstSpoor?.[1]?.SpoorNummer;

  await db("journey_event")
    .update({
      status: db.raw(
        'GREATEST("journey_event"."status", ?)',
        parseInt(arrivalStatus),
      ),
      ...(actualArrivalDatetime
        ? {
            arrival_time_actual: format(actualArrivalDatetime, "HH:mm:ss", {
              in: tz("Europe/Amsterdam"),
            }),
          }
        : {}),
      ...(actualArrivalPlatform
        ? { arrival_platform_actual: actualArrivalPlatform }
        : {}),
    })
    .whereIn("id", (knex) => {
      knex
        .select("journey_event.id")
        .from("journey_event")
        .innerJoin("journey", "journey_event.journey_id", "journey.id")
        .innerJoin("service", "journey.service_id", "service.id")
        .where("service.train_number", msg.RitId)
        .andWhere("journey.running_on", msg.RitDatum)
        .andWhere(
          "journey_event.station",
          msg.RitStation.StationCode.toLowerCase(),
        );
    });

  return Ok(void 0);
};
