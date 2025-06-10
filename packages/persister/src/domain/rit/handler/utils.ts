import { format as formatDate } from "date-fns";
import type { Knex } from "knex";
import { match, P } from "ts-pattern";

import { JourneyEventType } from "../../../types/db";
import { StopType } from "../types";
import type { Journey, JourneySegment, JourneySegmentStation } from "../types";

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
      [StopType.BoardingOnly, "J", P.string, P.nullish], // this makes no sense, but it does occur
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

export const inlineSplittingJourneys = (
  journeys: Journey[],
): JourneySegment[] => {
  if (journeys.length === 1) {
    return journeys[0].LogischeRitDeel;
  }

  const allJourneySegments = journeys.flatMap(
    (journey) => journey.LogischeRitDeel,
  );

  const segmentTrainNumbers = new Set(
    allJourneySegments.map((js) => js.LogischeRitDeelNummer),
  );

  if (allJourneySegments.length === segmentTrainNumbers.size) {
    // every journey segment has a unique train number, so we don't need to merge anything
    return allJourneySegments;
  }

  const mergedJourneySegments: JourneySegment[] = [];

  for (const trainNumber of segmentTrainNumbers) {
    const segments = allJourneySegments.filter(
      (js) => js.LogischeRitDeelNummer === trainNumber,
    );

    if (segments.length === 1) {
      mergedJourneySegments.push(segments[0]);
      continue;
    }

    const allStations = segments
      .flatMap((js) => js.LogischeRitDeelStation)
      .filter(
        (segment, index, segments) =>
          segments.findIndex(
            (s) => s.Station.StationCode === segment.Station.StationCode,
          ) === index,
      );

    const allChanges = segments
      .flatMap((js) => js.Wijziging)
      .filter((c) => c !== undefined);

    mergedJourneySegments.push({
      LogischeRitDeelNummer: trainNumber,
      LogischeRitDeelStation: allStations,
      Wijziging: allChanges.length > 0 ? allChanges : undefined,
    });
  }

  return mergedJourneySegments;
};
