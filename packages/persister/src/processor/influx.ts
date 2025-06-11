import type { Result } from "ts-results-es";
import type { WriteApi } from "@influxdata/influxdb-client";

import type { PosMessage } from "../domain/pos/types";
import { createInfluxClient } from "../influx";
import posHandler from "../domain/pos";

import type { InfoPlusMessage, Processor } from ".";

export type InfluxHandler<TMessage extends InfoPlusMessage> = (
  influx: WriteApi,
  message: TMessage,
) => Promise<Result<any, string>>;

export abstract class InfluxProcessor<TMessage extends InfoPlusMessage>
  implements Processor<TMessage>
{
  constructor(
    protected readonly influx: WriteApi,
    protected readonly handler: InfluxHandler<TMessage>,
  ) {}

  async processMessage(message: TMessage) {
    return await this.handler(this.influx, message);
  }
}

export class PosProcessor extends InfluxProcessor<PosMessage> {
  static build(): Promise<PosProcessor> {
    const { write } = createInfluxClient();
    return Promise.resolve(new PosProcessor(write));
  }

  constructor(protected readonly influx: WriteApi) {
    super(influx, posHandler);
  }
}
