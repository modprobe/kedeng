package io.t8n.kedeng.api.rest

import io.ktor.server.plugins.BadRequestException
import io.ktor.server.plugins.NotFoundException
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.get
import io.ktor.server.routing.route
import io.ktor.server.sse.heartbeat
import io.ktor.server.sse.send
import io.ktor.server.sse.sse
import io.ktor.sse.ServerSentEvent
import io.ktor.sse.TypedServerSentEvent
import io.ktor.util.reflect.reifiedType
import io.t8n.kedeng.TrainPositions
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.serializer
import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

fun Route.trainPositionRoutes(trainPositions: TrainPositions) {
    route("/position") {
        get {
            val positions = trainPositions.findLatestForAllRollingStock()
            call.respond(positions)
        }

        get(Regex("""(?<range>P.+)""")) {
            val range = call.parameters["range"] ?: throw BadRequestException("range is required")
            val parsedRange = Duration.parseOrNull(range) ?: throw BadRequestException("invalid range")

            val positions = trainPositions.findLatestForAllRollingStock(parsedRange)
            call.respond(positions)
        }

        get(Regex("""(?<rollingStockNumber>\d+)""")) {
            val rollingStockNumber =
                call.parameters["rollingStockNumber"]
                    ?: throw BadRequestException("rolling stock number is required")

            val position =
                trainPositions.findLatestByRollingStockNumber(rollingStockNumber)
                    ?: throw NotFoundException("position not found")

            call.respond(position)
        }

        sse("/stream", serialize = { typeInfo, it ->
            val serializer = Json.serializersModule.serializer(typeInfo.kotlinType!!)
            Json.encodeToString(serializer, it)
        }) {
            heartbeat {
                period = 10.seconds
                event = ServerSentEvent(event = "heartbeat")
            }

            val sendFull: suspend () -> Unit =
                { -> send(trainPositions.findLatestForAllRollingStock(1.hours), event = "full") }

            val sendDelta: suspend () -> Unit =
                { -> send(trainPositions.findLatestForAllRollingStock(15.seconds), event = "delta") }

            // send a full update (all positions from the last hour) followed by 3 overlapping deltas every 10 seconds
            runCatching {
                while (true) {
                    sendFull()
                    delay(10.seconds)
                    repeat(3) {
                        sendDelta()
                        delay(10.seconds)
                    }
                }
            }.also { close() }
        }
    }
}