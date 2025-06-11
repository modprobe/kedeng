import { InfluxDB } from "@influxdata/influxdb-client";

import { requireEnv } from "./utils";

export const createInfluxClient = () => {
  const client = new InfluxDB({
    url: requireEnv("INFLUX_URL"),
    token: requireEnv("INFLUX_TOKEN"),
  });

  const write = client.getWriteApi(
    requireEnv("INFLUX_ORG"),
    requireEnv("INFLUX_BUCKET"),
  );

  const query = client.getQueryApi(requireEnv("INFLUX_ORG"));

  return { client, write, query };
};
