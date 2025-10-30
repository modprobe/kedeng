package io.t8n.kedeng.api.graphql

import com.apurebase.kgraphql.schema.dsl.SchemaBuilder
import kotlinx.datetime.LocalDate
import kotlinx.datetime.LocalTime
import kotlin.time.Duration
import kotlin.time.Instant
import kotlin.uuid.Uuid

fun SchemaBuilder.appScalars() {
    stringScalar<Uuid> {
        name = "UUID"
        deserialize = { uuid: String -> Uuid.parse(uuid) }
        serialize = Uuid::toString
    }

    stringScalar<LocalDate> {
        name = "Date"
        deserialize = { date: String -> LocalDate.parse(date) }
        serialize = LocalDate::toString
    }

    stringScalar<LocalTime> {
        name = "Time"
        deserialize = { time: String -> LocalTime.parse(time) }
        serialize = LocalTime::toString
    }

    stringScalar<Duration> {
        name = "Duration"
        deserialize = { duration: String -> Duration.parseIsoString(duration) }
        serialize = Duration::toIsoString
    }

    stringScalar<Instant> {
        name = "Instant"
        deserialize = { instant: String -> Instant.parse(instant) }
        serialize = Instant::toString
    }
}