import { configureSync, getConsoleSink } from "@logtape/logtape";
import { formatDate, parseISO } from "date-fns";

import type { DateISOString, DateTimeISOString } from "./types/infoplus";

export const requireEnv = (env: string): string => {
  const value = process.env[env];
  if (!value) {
    throw new Error(`Environment variable "${env}" is required`);
  }

  return value;
};

export const sleep = async (durationMs: number) =>
  new Promise((r) => setTimeout(r, durationMs));

export const setupLogger = () =>
  configureSync({
    sinks: {
      console: getConsoleSink({
        formatter: (record) => JSON.stringify(record) + "\n",
      }),
    },
    loggers: [
      { category: ["kedeng"], lowestLevel: "debug", sinks: ["console"] },
      { category: ["logtape", "meta"], sinks: [] },
    ],
  });

export function* circularIterator<T>(
  input: T[],
  iterationCount: number = Infinity,
): Generator<T> {
  let i = 0;

  for (i; i < iterationCount; i += 1) {
    yield input[i % input.length];
  }

  return i;
}

export const extractTimeFromIsoString = (
  input: DateTimeISOString,
): DateISOString => formatDate(parseISO(input), "HH:mm:ss");
