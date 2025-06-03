import fs from "node:fs/promises";

import { configureSync, getConsoleSink } from "@logtape/logtape";
import { parse, set } from "date-fns";

import type { Stream } from ".";

export const requireEnv = (env: string): string => {
  const value = process.env[env];
  if (!value) {
    throw new Error(`Environment variable "${env}" is required`);
  }

  return value;
};

export const sleep = async (durationMs: number) =>
  new Promise((r) => setTimeout(r, durationMs));

export const LOGGER_CATEGORY = ["kedeng", "persister"];

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

export const saveMessageToFile = async (message: object, stream: Stream) => {
  const path = `${__dirname}/../docs/${stream.toUpperCase()}/failed`;
  await fs.mkdir(path, { recursive: true });

  const fileName = `${crypto.randomUUID()}.json`;
  await fs.writeFile(`${path}/${fileName}`, JSON.stringify(message));
};

export const parseDateString = (input: string) =>
  parse(
    input,
    "yyyy-MM-dd",
    set(new Date(), { hours: 0, minutes: 0, seconds: 0, milliseconds: 0 }),
  );
