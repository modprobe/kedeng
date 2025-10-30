package io.t8n.kedeng.api.rest

import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.t8n.kedeng.api.dto.toView
import io.t8n.kedeng.database.entity.Service
import org.jetbrains.exposed.v1.jdbc.transactions.transaction

fun Route.serviceRoutes() {
    get("/service") {
        val services = transaction {
            Service.all().map(Service::toView)
        }
        call.respond(services)
    }
}

