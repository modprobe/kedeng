package io.t8n.kedeng.api.dto

import io.t8n.kedeng.database.entity.Journey as JourneyEntity
import kotlinx.datetime.LocalDate
import kotlinx.serialization.Serializable
import kotlin.uuid.Uuid
import kotlin.uuid.toKotlinUuid

@Serializable
data class Journey(
    val id: Uuid,
    val service: Service,
    val runningOn: LocalDate,
    val events: List<JourneyEvent>,
    val attributes: List<String>,
) {
    companion object {
        fun fromEntity(journey: JourneyEntity) = Journey(
            journey.id.value.toKotlinUuid(),
            Service.fromEntity(journey.service),
            journey.runningOn,
            journey.journeyEvents.map({ it.toView() }),
            journey.attributes.orEmpty(),
        )
    }
}

fun JourneyEntity.toView() = Journey.fromEntity(this)