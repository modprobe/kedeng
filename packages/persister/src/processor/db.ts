import type { Knex } from "knex";
import { Err, type Result } from "ts-results-es";
import { getLogger } from "@logtape/logtape";

import type { RitMessage } from "../domain/rit/types";
import type { DasMessage } from "../domain/das/types";
import type { DvsMessage } from "../domain/dvs/types";
import ritHandler from "../domain/rit";
import dasHandler from "../domain/das";
import dvsHandler from "../domain/dvs";
import { LOGGER_CATEGORY } from "../utils";
import { createDbConnection } from "../db";
import { Stream } from "..";

import type { InfoPlusMessage, Processor } from ".";

export type DbHandler<TMessage extends InfoPlusMessage> = (
  db: Knex,
  data: TMessage,
) => Promise<Result<any, string | Error>>;

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

  async processMessage(
    message: TMessage,
  ): Promise<Result<any, string | Error>> {
    const dbTransaction = await this.db.transaction();

    let result: Result<any, string | Error>;
    try {
      result = await this.processor(dbTransaction, message);
      await dbTransaction.commit();
    } catch (e: any) {
      if (!(e instanceof Error)) {
        result = Err("processing failed, unknown error");
      } else {
        result = Err(e);
      }
    }

    if (result.isErr()) {
      this.logger.error("processing failed", {
        stream: this.sourceStream,
        err: result.unwrapErr(),
      });
      await dbTransaction.rollback();
    }

    return result;
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
