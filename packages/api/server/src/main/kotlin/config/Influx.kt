package io.t8n.kedeng.config

import com.influxdb.client.InfluxDBClientOptions

fun influxConfig() = InfluxDBClientOptions.builder()
    .url(Environment.get("INFLUX_URL"))
    .bucket(Environment.get("INFLUX_BUCKET"))
    .org(Environment.get("INFLUX_ORG"))
    .authenticateToken(Environment.get("INFLUX_TOKEN").toCharArray())
    .build()