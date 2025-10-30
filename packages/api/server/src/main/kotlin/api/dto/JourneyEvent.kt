package io.t8n.kedeng.api.dto

import io.t8n.kedeng.PlannedActualString
import io.t8n.kedeng.PlannedActualTime
import io.t8n.kedeng.database.entity.JourneyEvent as JourneyEventEntity
import kotlinx.datetime.toJavaLocalTime
import kotlinx.serialization.Serializable
import kotlin.time.Duration
import kotlin.time.toKotlinDuration
import kotlin.uuid.Uuid
import kotlin.uuid.toKotlinUuid

@Serializable
data class PlannedActualTimeWithDelay(
    val planned: LocalTime?,
    val actual: LocalTime?,
) {
    val delay: Duration? = if (planned != null && actual != null) {
        java.time.Duration.between(planned.toJavaLocalTime(), actual.toJavaLocalTime()).toKotlinDuration()
    } else null

    companion object {
        fun fromGeneric(prior: PlannedActualTime) = PlannedActualTimeWithDelay(
            prior.planned,
            prior.actual,
        )
    }

    fun isNull() = planned == null && actual == null
    fun collapseToNull() = if (isNull()) null else this
}

@Serializable
data class TimeAndPlatform(
    val time: PlannedActualTimeWithDelay,
    val platform: PlannedActualString,
) {
    fun isNull() = time.isNull() && platform.isNull()
    fun collapseToNull() = if (isNull()) null else this
}

@Serializable
data class JourneyEvent(
    val id: Uuid,
    val station: Station,
    val arrival: TimeAndPlatform?,
    val departure: TimeAndPlatform?,
    val rollingStock: List<RollingStock>,
    val eventType: PlannedActualString,
) {
    companion object {
        fun fromEntity(journeyEvent: JourneyEventEntity) = JourneyEvent(
            journeyEvent.id.value.toKotlinUuid(),
            Station.fromEntity(journeyEvent.station),
            TimeAndPlatform(
                PlannedActualTimeWithDelay.fromGeneric(journeyEvent.arrivalTime),
                journeyEvent.arrivalPlatform,
            ).collapseToNull(),
            TimeAndPlatform(
                PlannedActualTimeWithDelay.fromGeneric(journeyEvent.departureTime),
                journeyEvent.departurePlatform,
            ).collapseToNull(),
            journeyEvent.rollingStock.map({ it.toView() }),
            journeyEvent.eventType,
        )
    }
}

fun JourneyEventEntity.toView() = JourneyEvent.fromEntity(this)