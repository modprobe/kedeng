package io.t8n.kedeng.config

import io.ktor.server.application.*
import io.ktor.server.plugins.cors.routing.*
import io.ktor.server.sse.*

fun Application.configureHTTP() {
    install(CORS) {
        allowNonSimpleContentTypes = true

        allowHeaders { true }
        anyMethod()
        anyHost()

    }
    install(SSE)
}
