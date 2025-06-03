import { tz } from "@date-fns/tz";
import { format as formatDate, parseISO } from "date-fns";

import { ChangeLevel } from "./handler";
import type {
  RitMessage,
  Journey,
  JourneySegment,
  JourneySegmentStation,
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
