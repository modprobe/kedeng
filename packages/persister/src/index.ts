import { getLogger } from "@logtape/logtape";
import { match } from "ts-pattern";
import type { Result } from "ts-results-es";
import { Err } from "ts-results-es";

import { createDbConnection } from "./db";
import { createJetstreamConnection, setupConsumers } from "./nats";
import { circularIterator, requireEnv, setupLogger } from "./utils";
import dvsHandler from "./domain/dvs";
import dasHandler from "./domain/das";
import ritHandler from "./domain/rit";
import type { Handler } from "./types";
import { setupParser } from "./parser";

setupLogger();
const logger = getLogger(["kedeng", "persister"]);

let shuttingDown = false;
["SIGTERM", "SIGINT", "beforeExit"].forEach((evt) =>
  process.on(evt, () => {
    shuttingDown = true;
  }),
);

export enum Stream {
  DAS = "DAS",
  DVS = "DVS",
  POS = "POS",
  RIT = "RIT",
}

export const allStreams = Object.values(Stream);

const toStream = (input?: string): Stream => {
  input = input ?? requireEnv("NATS_STREAM");
  if (!(input in Stream)) {
    throw new Error("Unknown stream specified in NATS_STREAM");
  }

  return Stream[input as keyof typeof Stream];
};

const noopHandler: (stream: Stream) => Handler<any> =
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  (stream) => (_db, _data) =>
    new Promise(() => {
      console.log(`noop for stream ${stream}`);
      return Err("not implemented");
    });

const getHandler = (stream: Stream): Handler<any> =>
  match(stream)
    .with(Stream.DVS, () => dvsHandler)
    .with(Stream.DAS, () => dasHandler)
    .with(Stream.POS, () => noopHandler(Stream.POS))
    .with(Stream.RIT, () => ritHandler)
    .run();

void (async () => {
  const db = await createDbConnection();
  const { nc, js, jsm } = await createJetstreamConnection();

  const parser = setupParser();

  const consumers = await setupConsumers(js, jsm, [
    Stream.DAS,
    Stream.DVS,
    Stream.RIT,
  ]);

  logger.info("Set up consumers", { consumers });

  for (const [stream, consumer] of circularIterator(
    Array.from(consumers.entries()),
  )) {
    if (shuttingDown) {
      logger.info("Shutting down...");
      await nc.close();
      break;
    }

    try {
      const message = await consumer.next();
      if (message === null) {
        logger.info(`No message received from stream ${stream}`);
        continue;
      }

      logger.debug("Received message from {stream}", { stream });

      const messageContent = message.string();
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const messageData = parser.parse(messageContent);

      logger.debug(`Parsed message from ${stream}`);

      const handleMessage = getHandler(toStream(message.info.stream));

      let result: Result<void, string>;
      try {
        result = await handleMessage(db, messageData);
      } catch (e: any) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
        result = Err(e.toString());
      }

      if (result.isOk()) {
        logger.info(`Successfully processed message from ${stream}`);
        message.ack();
      } else {
        logger.error(`failed to handle message: ${result.error}`);
        message.nak(5_000);
      }
    } catch (err: any) {
      logger.error(`failed to handle message: ${err}`, { err });
    }
  }
})();
