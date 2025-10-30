package io.t8n.kedeng.api.graphql

import com.apurebase.kgraphql.schema.dsl.SchemaBuilder
import io.ktor.server.plugins.*
import io.t8n.kedeng.TrainPositions
import io.t8n.kedeng.api.dto.RollingStock
import io.t8n.kedeng.api.dto.toView
import io.t8n.kedeng.api.dto.Journey as JourneyView
import io.t8n.kedeng.database.entity.Journey
import io.t8n.kedeng.database.table.JourneyTable
import io.t8n.kedeng.database.table.ServiceTable
import kotlinx.datetime.LocalDate
import org.jetbrains.exposed.v1.core.and
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.jdbc.select
import org.jetbrains.exposed.v1.jdbc.transactions.transaction
import kotlin.time.Duration.Companion.hours

fun SchemaBuilder.journeyConfig(trainPositions: TrainPositions) {
    type<JourneyView> {
        name = "Journey"
        property("position") {
            resolver { journey -> trainPositions.findLatestByTrainNumber(journey.service.trainNumber, 1.hours) }
        }
    }

    type<RollingStock> {
        property("position") {
            resolver { rs -> trainPositions.findLatestByRollingStockNumber(rs.materialNumber) }
        }
    }

    query("journey") {
        resolver { trainNumber: String, runningOn: LocalDate ->
            val result = transaction {
                val query = JourneyTable.innerJoin(ServiceTable).select(JourneyTable.columns).where {
                    (ServiceTable.trainNumber eq trainNumber) and (JourneyTable.runningOn eq runningOn)
                }.withDistinct().firstOrNull()

                if (query == null) {
                    return@transaction null
                }

                Journey.wrapRow(query).toView()
            }

            result
        }
    }
}