package io.t8n.kedeng

import com.influxdb.client.kotlin.InfluxDBClientKotlin
import com.influxdb.query.dsl.Flux
import com.influxdb.query.dsl.functions.restriction.Restrictions
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.Serializable
import kotlin.time.*
import kotlin.time.Duration.Companion.days

class TrainPositions(influxClient: InfluxDBClientKotlin, private val bucket: String) {
    companion object {
        private const val MEASUREMENT = "train_position"
        private const val TAG_ROLLING_STOCK_NUMBER = "rolling_stock_number"
    }

    @Serializable
    data class Position(
        val rollingStockNumber: String,
        val trainNumber: String?,
        val latitude: Double,
        val longitude: Double,
        val speed: Double,
        val direction: Double,
        val time: Instant,
    )

    private val influxQuery = influxClient.getQueryKotlinApi()

    suspend fun findLatestByRollingStockNumber(rsNumber: String, range: Duration = 7.days): Position? {
        val query = Flux.from(bucket)
            .range(calculateStart(range))
            .filter(
                Restrictions.and(
                    Restrictions.measurement().equal(MEASUREMENT),
                    Restrictions.tag(TAG_ROLLING_STOCK_NUMBER).equal(rsNumber)
                )
            )
            .pivot(arrayOf("_time"), arrayOf("_field"), "_value")
            .groupBy(arrayOf(TAG_ROLLING_STOCK_NUMBER))
            .sort(listOf("_time"), true)
            .limit(1)

        println(query.toString())
        val result = influxQuery.query(query.toString()).receiveCatching().getOrNull() ?: return null

        return Position(
            rsNumber,
            result.getValueByKey("train_number") as String,
            result.getValueByKey("latitude") as Double,
            result.getValueByKey("longitude") as Double,
            result.getValueByKey("speed") as Double,
            result.getValueByKey("direction") as Double,
            result.time!!.toKotlinInstant(),
        )
    }

    suspend fun findLatestByTrainNumber(trainNumber: String, range: Duration = 7.days): List<Position> {
        val query = Flux.from(bucket)
            .range(calculateStart(range))
            .filter(Restrictions.measurement().equal(MEASUREMENT))
            .pivot(arrayOf("_time"), arrayOf("_field"), "_value")
            .filter(Restrictions.tag("train_number").equal(trainNumber))
            .groupBy(arrayOf(TAG_ROLLING_STOCK_NUMBER))
            .sort(listOf("_time"), true)
            .limit(1)

        val results = mutableListOf<Position>()

        for (result in influxQuery.query(query.toString())) {
            results.add(
                Position(
                    result.getValueByKey(TAG_ROLLING_STOCK_NUMBER) as String,
                    result.getValueByKey("train_number") as? String,
                    result.getValueByKey("latitude") as Double,
                    result.getValueByKey("longitude") as Double,
                    result.getValueByKey("speed") as Double,
                    result.getValueByKey("direction") as Double,
                    result.time!!.toKotlinInstant(),
                )
            )
        }


        return results
    }

    suspend fun findLatestForAllRollingStock(range: Duration = 7.days): List<Position> {
        val query = Flux.from(bucket)
            .range(calculateStart(range))
            .filter(Restrictions.measurement().equal(MEASUREMENT))
            .pivot(arrayOf("_time"), arrayOf("_field"), "_value")
            .groupBy(arrayOf(TAG_ROLLING_STOCK_NUMBER))
            .sort(listOf("_time"), true)
            .limit(1)

        println(query.toString())
        val results = mutableListOf<Position>()

        for (result in influxQuery.query(query.toString())) {
            results.add(
                Position(
                    result.getValueByKey(TAG_ROLLING_STOCK_NUMBER) as String,
                    result.getValueByKey("train_number") as? String,
                    result.getValueByKey("latitude") as Double,
                    result.getValueByKey("longitude") as Double,
                    result.getValueByKey("speed") as Double,
                    result.getValueByKey("direction") as Double,
                    result.time!!.toKotlinInstant(),
                )
            )
        }

        return results

    }

    private fun calculateStart(range: Duration) = when (range.isPositive()) {
        true -> Clock.System.now() - range
        false -> Clock.System.now() + range
    }.toJavaInstant()
}