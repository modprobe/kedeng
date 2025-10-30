package io.t8n.kedeng.database.entity

import io.t8n.kedeng.database.table.RollingStockTable
import org.jetbrains.exposed.v1.core.dao.id.CompositeID
import org.jetbrains.exposed.v1.core.dao.id.EntityID
import org.jetbrains.exposed.v1.dao.CompositeEntity
import org.jetbrains.exposed.v1.dao.ImmutableEntityClass

class RollingStock(id: EntityID<CompositeID>) : CompositeEntity(id) {
    companion object : ImmutableEntityClass<CompositeID, RollingStock>(RollingStockTable)

    val journey by Journey referencedOn RollingStockTable.journey
    val journeyEvent by JourneyEvent referencedOn RollingStockTable.journeyEvent

    val departureOrder by RollingStockTable.departureOrder

    val materialType by RollingStockTable.materialType
    val materialSubtype by RollingStockTable.materialSubtype
    val materialNumber by RollingStockTable.materialNumber
}