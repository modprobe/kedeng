package io.t8n.kedeng.config

import io.ktor.server.application.Application
import org.jetbrains.exposed.v1.jdbc.Database

fun Application.configureDatabase(): Database {
    val host = Environment.get("DB_HOST")
    val port = Environment.get("DB_PORT", "5432")
    val name = Environment.get("DB_NAME")

    return Database.connect(
        "jdbc:postgresql://$host:$port/$name",
        driver = "org.postgresql.Driver",
        user = Environment.get("DB_USER"),
        password = Environment.get("DB_PASSWORD")
    )
}