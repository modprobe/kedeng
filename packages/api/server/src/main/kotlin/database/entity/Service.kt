package io.t8n.kedeng.database.entity

import io.t8n.kedeng.database.table.JourneyTable
import io.t8n.kedeng.database.table.ServiceTable
import org.jetbrains.exposed.v1.core.dao.id.EntityID
import org.jetbrains.exposed.v1.dao.ImmutableEntityClass
import org.jetbrains.exposed.v1.dao.UUIDEntity
import java.util.UUID


class Service(id: EntityID<UUID>) : UUIDEntity(id) {
    companion object : ImmutableEntityClass<UUID, Service>(ServiceTable)

    val provider by ServiceTable.provider
    val trainType by ServiceTable.trainType
    val trainNumber by ServiceTable.trainNumber
    val timetableYear by ServiceTable.timetableYear

    val journeys by Journey referrersOn JourneyTable.service
}
