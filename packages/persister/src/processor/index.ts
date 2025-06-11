import { Ok, type Result } from "ts-results-es";
import { getLogger } from "@logtape/logtape";

import type { DasMessage } from "../domain/das/types";
import type { DvsMessage } from "../domain/dvs/types";
import type { RitMessage } from "../domain/rit/types";
import type { PosMessage } from "../domain/pos/types";
import type { Stream } from "..";
import { LOGGER_CATEGORY } from "../utils";

export interface Processor<TMessage extends InfoPlusMessage> {
  processMessage: (message: TMessage) => Promise<Result<any, string>>;
}

export type InfoPlusMessage = RitMessage | DasMessage | DvsMessage | PosMessage;

export class NoopProcessor<TMessage extends InfoPlusMessage>
  implements Processor<TMessage>
{
  private logger = getLogger([...LOGGER_CATEGORY]);

  constructor(private readonly stream: Stream) {}

  processMessage(): Promise<Result<any, string>> {
    this.logger.warn(`Unhandled message from ${this.stream}`);
    return Promise.resolve(Ok(undefined));
  }
}
