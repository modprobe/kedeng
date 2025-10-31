import { getLogger } from "@logtape/logtape";
import { match } from "ts-pattern";
import type { Result } from "ts-results-es";

import { createJetstreamConnection, setupConsumer } from "./nats";
import { requireEnv, setupLogger, sleep } from "./utils";
import { setupParser } from "./parser";
import type { Processor } from "./processor";
import { DasProcessor, DvsProcessor, RitProcessor } from "./processor/db";
import { PosProcessor } from "./processor/influx";

setupLogger();
const logger = getLogger(["kedeng", "persister"]);

let shuttingDown = false;
["SIGTERM", "SIGINT", "beforeExit"].forEach((evt) => {
  process.on(evt, () => {
    shuttingDown = true;
  });
});

export enum Stream {
  DAS = "DAS",
  DVS = "DVS",
  POS = "POS",
  RIT = "RIT",
}

export const allStreams = Object.values(Stream);

const toStream = (input: string): Stream => {
  if (!(input in Stream)) {
    throw new Error("Unknown stream specified in NATS_STREAM");
  }

  return Stream[input as keyof typeof Stream];
};

const getProcessor = async (stream: Stream): Promise<Processor<any>> =>
  match(stream)
    .with(Stream.DVS, () => DvsProcessor.build())
    .with(Stream.DAS, () => DasProcessor.build())
    .with(Stream.RIT, () => RitProcessor.build())
    .with(Stream.POS, () => PosProcessor.build())
    .exhaustive();

void (async () => {
  const { nc, js, jsm } = await createJetstreamConnection();

  const sourceStream = toStream(requireEnv("KEDENG_PERSISTER_NATS_STREAM"));
  const processor = await getProcessor(sourceStream);

  const parser = setupParser();

  const consumer = await setupConsumer(js, jsm, sourceStream);
  logger.info("Set up consumer", { consumer });

  do {
    try {
      const message = await consumer.next();
      if (message === null) {
        logger.info(`No message received from stream ${sourceStream}`);
        await sleep(1_000);
        continue;
      }

      logger.debug(`Received message from ${sourceStream}`, { sourceStream });

      const messageContent = message.string();
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const messageData = parser.parse(messageContent);

      logger.debug(`Parsed message from ${sourceStream}`);

      const result: Result<any, string | Error> =
        await processor.processMessage(messageData);

      if (result.isOk()) {
        logger.info(`Successfully processed message from ${sourceStream}`);
        message.ack();
      } else {
        logger.error(`failed to handle message: ${result.error}`);
        message.nak(5_000);
      }
    } catch (err: any) {
      logger.error(`failed to handle message: ${err}`, { err });
    }
  } while (!shuttingDown);

  logger.info("Shutting down...");
  await nc.close();
})();
