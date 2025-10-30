package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.dao.id.UUIDTable
import org.jetbrains.exposed.v1.datetime.time

object JourneyEventTable : UUIDTable("journey_event") {
    val journey = reference("journey_id", JourneyTable)
    val station = reference("station", StationTable)

    val eventTypePlanned = text("event_type_planned").nullable()
    val eventTypeActual = text("event_type_actual").nullable()

    val arrivalTimePlanned = time("arrival_time_planned").nullable()
    val arrivalTimeActual = time("arrival_time_actual").nullable()
    val arrivalPlatformPlanned = text("arrival_platform_planned").nullable()
    val arrivalPlatformActual = text("arrival_platform_actual").nullable()

    val departureTimePlanned = time("departure_time_planned").nullable()
    val departureTimeActual = time("departure_time_actual").nullable()
    val departurePlatformPlanned = text("departure_platform_planned").nullable()
    val departurePlatformActual = text("departure_platform_actual").nullable()

    val status = integer("status")
    val departureCancelled = bool("departure_cancelled").nullable()
    val arrivalCancelled = bool("arrival_cancelled").nullable()

    val attributes = array<String>("attributes").nullable()
}