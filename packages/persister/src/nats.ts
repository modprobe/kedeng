import { connect } from "@nats-io/transport-node";
import type {
  Consumer,
  JetStreamClient,
  JetStreamManager,
} from "@nats-io/jetstream";
import {
  AckPolicy,
  DeliverPolicy,
  jetstream,
  jetstreamManager,
  JetStreamApiError,
  JetStreamApiCodes,
} from "@nats-io/jetstream";
import { nanos, type NatsConnection } from "@nats-io/nats-core";
import { getLogger } from "@logtape/logtape";

import { requireEnv } from "./utils";

import type { Stream } from "./index";
import { allStreams } from "./index";

const logger = getLogger(["kedeng", "persister", "nats"]);

export const createNatsConnection = (): Promise<NatsConnection> =>
  connect({
    servers: `${requireEnv("NATS_HOST")}:${requireEnv("NATS_PORT")}`,
    user: process.env.NATS_USER ?? undefined,
    pass: process.env.NATS_PASSWORD ?? undefined,
  });

export const createJetstreamConnection = async (): Promise<{
  nc: NatsConnection;
  js: JetStreamClient;
  jsm: JetStreamManager;
}> => {
  const connection = await createNatsConnection();
  return {
    nc: connection,
    js: jetstream(connection),
    jsm: await jetstreamManager(connection),
  };
};

export const consumerName = (stream: Stream): string =>
  `kedeng-persister-${stream}`;

export const setupConsumers = async (
  js: JetStreamClient,
  jsm: JetStreamManager,
  enabledStreams: Stream[] = allStreams,
): Promise<Map<Stream, Consumer>> => {
  const consumers: Map<Stream, Consumer> = new Map();

  for (const stream of enabledStreams) {
    const name = consumerName(stream);

    try {
      const consumer = await js.consumers.get(stream, name);
      logger.info(`Consumer "${name}" already exists`, {
        info: consumer.info(true),
      });
      consumers.set(stream, consumer);
      continue;
    } catch (err: any) {
      if (
        !(err instanceof JetStreamApiError) ||
        err.code !== JetStreamApiCodes.ConsumerNotFound
      ) {
        logger.error("Error checking for existing consumer", { err });
        continue;
      }

      logger.info(`Consumer "${name}" not found yet, creating...`);
      await jsm.consumers.add(stream, {
        durable_name: name,
        ack_policy: AckPolicy.Explicit,
        max_ack_pending: -1,
        ack_wait: nanos(5_000),
        deliver_policy: DeliverPolicy.All,
        inactive_threshold: nanos(300_000),
        max_deliver: 10,
        backoff: [nanos(5_000)],
      });

      const consumer = await js.consumers.get(stream, name);
      consumers.set(stream, consumer);
    }
  }

  return consumers;
};
