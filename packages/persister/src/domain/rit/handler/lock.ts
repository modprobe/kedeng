import { randomUUID } from "crypto";

import type { UTCDate } from "@date-fns/utc";
import { hoursToSeconds, minutesToSeconds } from "date-fns";
import { type Result, Ok, Err } from "ts-results-es";
import type { RedisClientType } from "@redis/client";

import { createRedisClient } from "../../../redis";
import type { DateISOString } from "../../../types/infoplus";

export const isNewestMessage = async (
  trainNumber: string,
  runningOn: DateISOString,
  timestamp: UTCDate,
): Promise<Result<undefined, string>> => {
  const redis = await createRedisClient();
  const key = `ritLastUpdated:${trainNumber}:${runningOn}`;

  const updateToNewTimestamp = async () => {
    await redis.set(key, timestamp.valueOf(), {
      expiration: { type: "EX", value: hoursToSeconds(6) },
    });
  };

  const parseLatest = async () => {
    const latest = await redis.get(key);
    if (latest === null) {
      return null;
    }

    return parseInt(latest);
  };

  const latest = await parseLatest();
  if (latest === null) {
    await updateToNewTimestamp();
    return Ok(undefined);
  }

  if (latest < timestamp.valueOf()) {
    await updateToNewTimestamp();
    return Ok(undefined);
  }

  return Err("this is not the most recent message we processed for this train");
};

export const enum RitProcessingLockErrors {
  // acquire
  ALREADY_LOCKED_BY_US = "Lock is already acquired by this instance",
  ALREADY_LOCKED_BY_SOMEONE_ELSE = "Lock is already acquired by another instance",
  // release
  NOT_LOCKED = "Lock is not acquired, so cannot be released",
  LOCK_UPDATED_BY_OTHER_INSTANCE = "Lock was released and re-acquired by another instance",
}

export class RitProcessingLock {
  private static readonly KEY_PREFIX = "ritLock";
  private static readonly LOCK_TIMEOUT = minutesToSeconds(5);

  private readonly lockValue: string;

  static async build(
    trainNumber: string,
    runningOn: DateISOString,
    redis?: RedisClientType,
  ): Promise<RitProcessingLock> {
    redis = redis ?? (await createRedisClient());

    const key = `${this.KEY_PREFIX}:${trainNumber}:${runningOn}`;
    return new this(key, redis);
  }

  private constructor(
    private readonly lockKey: string,
    private readonly redis: RedisClientType,
    lockValue?: string,
  ) {
    this.lockValue = lockValue ?? process.env?.REDIS_LOCK_VALUE ?? randomUUID();
  }

  async isLocked(): Promise<boolean> {
    const value = await this.redis.get(this.lockKey);
    return value !== null && value === this.lockValue;
  }

  async acquire(): Promise<Result<undefined, RitProcessingLockErrors>> {
    const alreadyLocked = await this.isLocked();
    if (alreadyLocked) {
      return Err(RitProcessingLockErrors.ALREADY_LOCKED_BY_US);
    }

    const created = await this.redis.set(this.lockKey, this.lockValue, {
      condition: "NX",
      expiration: { type: "EX", value: RitProcessingLock.LOCK_TIMEOUT },
    });

    if (created === null) {
      return Err(RitProcessingLockErrors.ALREADY_LOCKED_BY_SOMEONE_ELSE);
    }

    return Ok(undefined);
  }

  async release(): Promise<Result<undefined, RitProcessingLockErrors>> {
    const isLocked = await this.isLocked();
    if (!isLocked) {
      return Err(RitProcessingLockErrors.NOT_LOCKED);
    }

    const value = await this.redis.get(this.lockKey);
    if (value !== this.lockValue) {
      return Err(RitProcessingLockErrors.LOCK_UPDATED_BY_OTHER_INSTANCE);
    }

    await this.redis.del(this.lockKey);
    return Ok(undefined);
  }

  async clearRegardless(): Promise<void> {
    await this.redis.del(this.lockKey);
  }
}
