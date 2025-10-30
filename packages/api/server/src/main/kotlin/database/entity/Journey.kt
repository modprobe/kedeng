package io.t8n.kedeng.database.entity

import io.t8n.kedeng.database.table.JourneyEventTable
import io.t8n.kedeng.database.table.JourneyTable
import org.jetbrains.exposed.v1.core.dao.id.EntityID
import org.jetbrains.exposed.v1.dao.ImmutableEntityClass
import org.jetbrains.exposed.v1.dao.UUIDEntity
import java.util.UUID

class Journey(id: EntityID<UUID>) : UUIDEntity(id) {
    companion object : ImmutableEntityClass<UUID, Journey>(JourneyTable)

    val service by Service referencedOn JourneyTable.service
    val runningOn by JourneyTable.runningOn
    val attributes by JourneyTable.attributes

    val journeyEvents by JourneyEvent referrersOn JourneyEventTable.journey
}