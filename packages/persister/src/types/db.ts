import type { TimeString } from "./infoplus";

export const enum JourneyEventType {
  Passage = "PASSAGE",
  Departure = "DEPARTURE",
  ShortStop = "SHORT_STOP",
  LongerStop = "LONGER_STOP",
  Arrival = "ARRIVAL",
}
