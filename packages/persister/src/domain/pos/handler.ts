import { Ok } from "ts-results-es";
import { Point } from "@influxdata/influxdb-client";
import { parseISO } from "date-fns";

import type { InfluxHandler } from "../../processor/influx";

import type { PosMessage } from "./types";

export const handler: InfluxHandler<PosMessage> = async (influx, message) => {
  const locations = message.ArrayOfTreinLocation.TreinLocation;

  for (const location of locations) {
    const trainNumber = location.TreinNummer;

    const points = location.TreinMaterieelDelen.map((rollingStock) =>
      new Point("train_position")
        .timestamp(parseISO(rollingStock.GpsDatumTijd))
        .tag("train_number", trainNumber)
        .tag("rolling_stock_number", rollingStock.MaterieelDeelNummer)
        .tag("provider", "NS")
        .floatField("latitude", parseFloat(rollingStock.Latitude))
        .floatField("longitude", parseFloat(rollingStock.Longitude))
        .floatField("speed", parseFloat(rollingStock.Snelheid))
        .floatField("direction", parseFloat(rollingStock.Richting)),
    );

    influx.writePoints(points);
  }

  await influx.flush();

  return Ok(undefined);
};
