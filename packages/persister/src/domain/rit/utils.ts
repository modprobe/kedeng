import { tz } from "@date-fns/tz";
import { format as formatDate, parseISO } from "date-fns";
import { match, P } from "ts-pattern";
import type { Knex } from "knex";

import { JourneyEventType } from "../../types/db";

import { ChangeLevel } from "./handler";
import {
  type RitMessage,
  type Journey,
  type JourneySegment,
  type JourneySegmentStation,
  StopType,
} from "./types";

// converts an input ISO datetime string in UTC
// into the local time (HH:MM)
export const convertTime = (isoString: string): string =>
  formatDate(parseISO(isoString), "HH:mm:ss", { in: tz("Europe/Amsterdam") });

export const formatPlatform = ({
  SpoorNummer,
  SpoorFase,
}: {
  SpoorNummer: string;
  SpoorFase?: string;
}) => (SpoorFase ? `${SpoorNummer}${SpoorFase}` : SpoorNummer);

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

export const determineJourneyEventType = (
  stop: JourneySegmentStation,
): JourneyEventType => {
  return match([
    stop.StationnementType,
    stop.Stopt[1]?.text,
    stop.AankomstTijd?.[0]?.text,
    stop.VertrekTijd?.[0]?.text,
  ])
    .returnType<JourneyEventType>()
    .with(
      [StopType.Passage, P._, P._, P._],
      [StopType.ServiceStop, P._, P._, P._],
      [P._, "N", P._, P._],
      () => JourneyEventType.Passage,
    )
    .with(
      [StopType.Stop, "J", P.nullish, P.string],
      [StopType.BoardingOnly, "J", P.nullish, P.string],
      () => JourneyEventType.Departure,
    )
    .with(
      [StopType.Stop, "J", P.string, P.nullish],
      [StopType.AlightingOnly, "J", P.string, P.nullish],
      () => JourneyEventType.Arrival,
    )
    .with(
      [StopType.Stop, "J", P.string, P.string],
      [StopType.AlightingOnly, "J", P.string, P.string],
      [StopType.BoardingOnly, "J", P.string, P.string],
      ([_a, _b, arrival, departure]) =>
        arrival === departure
          ? JourneyEventType.ShortStop
          : JourneyEventType.LongerStop,
    )
    .run();
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
