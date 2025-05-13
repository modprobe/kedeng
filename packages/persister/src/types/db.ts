import type { TimeString } from "./infoplus";

export const enum JourneyEventType {
  Passage = "PASSAGE",
  Departure = "DEPARTURE",
  ShortStop = "SHORT_STOP",
  LongerStop = "LONGER_STOP",
  Arrival = "ARRIVAL",
}

export type JourneyEventTable = {
  event_type_planned?: JourneyEventType;
  event_type_actual?: JourneyEventType;

  arrival_time_planned?: TimeString;
  arrival_time_actual?: TimeString;
  arrival_platform_planned?: string;
  arrival_platform_actual?: string;

  departure_time_planned?: TimeString;
  departure_time_actual?: TimeString;
  departure_platform_planned?: string;
  departure_platform_actual?: string;

  is_cancelled?: boolean;
};
