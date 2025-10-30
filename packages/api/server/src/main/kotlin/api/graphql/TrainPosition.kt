package io.t8n.kedeng.api.graphql

import com.apurebase.kgraphql.schema.dsl.SchemaBuilder
import io.t8n.kedeng.TrainPositions
import kotlin.time.Duration.Companion.hours

fun SchemaBuilder.trainPositionConfig(trainPositions: TrainPositions) {
    query("trainMap") {
        resolver { -> trainPositions.findLatestForAllRollingStock(1.hours) }
    }

    query("rollingStockPosition") {
        resolver { rollingStockNumber: String -> trainPositions.findLatestByRollingStockNumber(rollingStockNumber) }
    }

    query("trainPosition") {
        resolver { trainNumber: String -> trainPositions.findLatestByTrainNumber(trainNumber) }
    }
}