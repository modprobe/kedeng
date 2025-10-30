package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.dao.id.CompositeIdTable

object RollingStockTable : CompositeIdTable("rolling_stock") {
    val journey = reference("journey_id", JourneyTable.id)
    val journeyEvent = reference("journey_event_id", JourneyEventTable.id)
    val departureOrder = integer("departure_order").entityId()

    val materialType = text("material_type")
    val materialSubtype = text("material_subtype")
    val materialNumber = text("material_number")

    init {
        addIdColumn(journeyEvent)
    }
}