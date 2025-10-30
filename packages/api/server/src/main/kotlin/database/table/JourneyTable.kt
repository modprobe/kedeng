package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.dao.id.UUIDTable
import org.jetbrains.exposed.v1.datetime.date

object JourneyTable : UUIDTable("journey") {
    val service = reference("service_id", ServiceTable)
    val runningOn = date("running_on")
    val attributes = array<String>("attributes").nullable()
}