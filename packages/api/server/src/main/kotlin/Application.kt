package io.t8n.kedeng

import dev.hayden.KHealth
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.netty.*
import io.t8n.kedeng.config.*
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.runBlocking
import org.jetbrains.exposed.v1.jdbc.transactions.transaction

fun main(args: Array<String>): Unit = runBlocking {
    val mainServer = embeddedServer(Netty, port = 8080) {
        apiModule()
    }
    val healthServer = embeddedServer(Netty, port = 8081) {
        configureDatabase()
        install(KHealth) {
            readyChecks {
                check("database") {
                    try {
                        transaction {
                            exec("SELECT 1;") {}
                        }
                        true
                    } catch (e: Exception) {
                        false
                    }
                }
            }
        }
    }

    mainServer.start(wait = false)
    healthServer.start(wait = false)

    awaitCancellation()
}

fun Application.apiModule() {
    configureDI()
    configureHTTP()
    configureSerialization()
    configureRouting()
    configureDatabase()
    configureGraphQL()
}
