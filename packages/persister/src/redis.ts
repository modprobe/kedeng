import { createClient } from "@redis/client";

import { requireEnv } from "./utils";

export const createRedisClient = async () => {
  const client = createClient({
    socket: {
      host: requireEnv("REDIS_HOST"),
      port: parseInt(requireEnv("REDIS_PORT")),
    },
  });

  return client.connect();
};
