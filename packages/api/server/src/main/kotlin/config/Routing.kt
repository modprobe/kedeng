package io.t8n.kedeng.config

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.plugins.di.*
import io.ktor.server.plugins.statuspages.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import io.t8n.kedeng.TrainPositions
import io.t8n.kedeng.api.rest.journeyRoutes
import io.t8n.kedeng.api.rest.serviceRoutes
import io.t8n.kedeng.api.rest.trainPositionRoutes
import kotlinx.serialization.Serializable

fun Application.configureRouting() {
    install(StatusPages) {
        exception<Throwable> { call, cause ->
            val statusCode = defaultExceptionStatusCode(cause) ?: HttpStatusCode.InternalServerError

            @Serializable
            data class Error(
                val status: Int,
                val message: String,
            )

            call.respond(
                Error(statusCode.value, cause.message ?: "Internal Server Error")
            )
        }
    }

    val trainPositions: TrainPositions by dependencies
    routing {
        route("/v1") {
            serviceRoutes()
            journeyRoutes()
            trainPositionRoutes(trainPositions)
        }
    }
}
