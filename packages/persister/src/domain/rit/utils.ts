import { tz } from "@date-fns/tz";
import { format as formatDate, parseISO } from "date-fns";

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
