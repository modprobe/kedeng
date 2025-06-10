import type { Knex } from "knex";
import { Err, Ok, type Result } from "ts-results-es";
import { getLogger } from "@logtape/logtape";

import type { RitMessage } from "./domain/rit/types";
import type { DasMessage } from "./domain/das/types";
import type { DvsMessage } from "./domain/dvs/types";
import ritHandler from "./domain/rit";
import dasHandler from "./domain/das";
import dvsHandler from "./domain/dvs";
import { LOGGER_CATEGORY } from "./utils";
import { createDbConnection } from "./db";

import { Stream } from ".";

type InfoPlusMessage = RitMessage | DasMessage | DvsMessage;

export type DbHandler<TMessage extends InfoPlusMessage> = (
  db: Knex,
  data: TMessage,
) => Promise<Result<any, string>>;

export interface Processor<TMessage extends InfoPlusMessage> {
  processMessage: (message: TMessage) => Promise<Result<any, string>>;
}

abstract class DbProcessor<TMessage extends InfoPlusMessage>
  implements Processor<TMessage>
{
  private logger;

  constructor(
    protected readonly db: Knex,
    protected readonly sourceStream: Stream,
    protected readonly processor: DbHandler<TMessage>,
  ) {
    this.logger = getLogger([...LOGGER_CATEGORY, sourceStream]);
  }

  async processMessage(message: TMessage): Promise<Result<any, string>> {
    const dbTransaction = await this.db.transaction();

    let result: Result<any, string>;
    try {
      result = await this.processor(dbTransaction, message);
      if (result.isErr()) {
        throw new Error(result.unwrapErr());
      }

      await dbTransaction.commit();
    } catch (e: any) {
      if (!(e instanceof Error)) {
        result = Err("");
      } else {
        this.logger.error("processing failed", {
          stream: this.sourceStream,
          err: {
            name: e.name,
            message: e.message,
            stack: e.stack,
            cause: e.cause,
          },
        });

        result = Err(e.message);
      }

      await dbTransaction.rollback();
    }

    return result;
  }
}

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

export class RitProcessor extends DbProcessor<RitMessage> {
  static async build(): Promise<RitProcessor> {
    const db = await createDbConnection();
    return new this(db);
  }

  constructor(protected readonly db: Knex) {
    super(db, Stream.RIT, ritHandler);
  }
}

export class DasProcessor extends DbProcessor<DasMessage> {
  static async build(): Promise<DasProcessor> {
    const db = await createDbConnection();
    return new this(db);
  }

  constructor(protected readonly db: Knex) {
    super(db, Stream.DAS, dasHandler);
  }
}

export class DvsProcessor extends DbProcessor<DvsMessage> {
  static async build(): Promise<DvsProcessor> {
    const db = await createDbConnection();
    return new this(db);
  }

  constructor(protected readonly db: Knex) {
    super(db, Stream.DVS, dvsHandler);
  }
}
