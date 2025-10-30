package io.t8n.kedeng.database.entity

import io.t8n.kedeng.PlannedActualTime
import io.t8n.kedeng.PlannedActualString
import io.t8n.kedeng.database.table.JourneyEventTable
import io.t8n.kedeng.database.table.RollingStockTable
import org.jetbrains.exposed.v1.core.dao.id.EntityID
import org.jetbrains.exposed.v1.dao.ImmutableEntityClass
import org.jetbrains.exposed.v1.dao.UUIDEntity
import java.util.UUID

class JourneyEvent(id: EntityID<UUID>) : UUIDEntity(id) {
    companion object : ImmutableEntityClass<UUID, JourneyEvent>(JourneyEventTable)

    val journey by Journey referencedOn JourneyEventTable.journey
    val station by Station referencedOn JourneyEventTable.station

    private val arrivalTimePlanned by JourneyEventTable.arrivalTimePlanned
    private val arrivalTimeActual by JourneyEventTable.arrivalTimeActual
    val arrivalTime get() = PlannedActualTime(arrivalTimePlanned, arrivalTimeActual)

    private val arrivalPlatformPlanned by JourneyEventTable.arrivalPlatformPlanned
    private val arrivalPlatformActual by JourneyEventTable.arrivalPlatformActual
    val arrivalPlatform get() = PlannedActualString(arrivalPlatformPlanned, arrivalPlatformActual)

    private val departureTimePlanned by JourneyEventTable.departureTimePlanned
    private val departureTimeActual by JourneyEventTable.departureTimeActual
    val departureTime get() = PlannedActualTime(departureTimePlanned, departureTimeActual)

    private val departurePlatformPlanned by JourneyEventTable.departurePlatformPlanned
    private val departurePlatformActual by JourneyEventTable.departurePlatformActual
    val departurePlatform get() = PlannedActualString(departurePlatformPlanned, departurePlatformActual)

    private val eventTypePlanned by JourneyEventTable.eventTypePlanned
    private val eventTypeActual by JourneyEventTable.eventTypeActual
    val eventType get() = PlannedActualString(eventTypePlanned, eventTypeActual)

    val status by JourneyEventTable.status
    val departureCancelled by JourneyEventTable.departureCancelled
    val arrivalCancelled by JourneyEventTable.arrivalCancelled

    val rollingStock by RollingStock referrersOn RollingStockTable.journeyEvent
}