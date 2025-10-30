package io.t8n.kedeng.config

import com.influxdb.client.InfluxDBClientOptions
import com.influxdb.client.kotlin.InfluxDBClientKotlin
import com.influxdb.client.kotlin.InfluxDBClientKotlinFactory
import io.ktor.server.application.Application
import io.ktor.server.plugins.di.dependencies
import io.ktor.server.plugins.di.resolve
import io.t8n.kedeng.TrainPositions

fun Application.configureDI() {
    dependencies {
        provide<InfluxDBClientOptions> { influxConfig() }
        provide<InfluxDBClientKotlin> { InfluxDBClientKotlinFactory.create(resolve<InfluxDBClientOptions>()) }
        provide<TrainPositions> { TrainPositions(resolve(), resolve<InfluxDBClientOptions>().bucket!!) }
    }
}