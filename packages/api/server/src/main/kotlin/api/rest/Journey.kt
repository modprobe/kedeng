package io.t8n.kedeng.api.rest

import io.ktor.server.plugins.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.t8n.kedeng.api.dto.toView
import io.t8n.kedeng.database.entity.Journey
import io.t8n.kedeng.database.table.JourneyTable
import io.t8n.kedeng.database.table.ServiceTable
import kotlinx.datetime.LocalDate
import org.jetbrains.exposed.v1.core.and
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.jdbc.select
import org.jetbrains.exposed.v1.jdbc.transactions.transaction
import java.util.*

fun Route.journeyRoutes() {
    get("/journey/{id}") {
        val id = call.parameters["id"] ?: throw BadRequestException("journey id is required")
        val result = transaction {
            Journey.findById(UUID.fromString(id))?.toView() ?: throw NotFoundException("journey not found")
        }

        call.respond(result)
    }

    get("/journey/{trainNumber}/{runningOn}") {
        val trainNumber = call.parameters["trainNumber"] ?: throw BadRequestException("train number is required")
        val runningOn =
            LocalDate.parse(call.parameters["runningOn"] ?: throw BadRequestException("running on is required"))

        val result = transaction {
            val query = JourneyTable.innerJoin(ServiceTable).select(JourneyTable.columns).where {
                (ServiceTable.trainNumber eq trainNumber) and (JourneyTable.runningOn eq runningOn)
            }.withDistinct().firstOrNull() ?: throw NotFoundException("journey not found")

            Journey.wrapRow(query).toView()
        }

        call.respond(result)
    }
}

