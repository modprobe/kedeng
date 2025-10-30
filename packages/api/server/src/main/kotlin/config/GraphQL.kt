package io.t8n.kedeng.config

import com.apurebase.kgraphql.GraphQL
import io.ktor.server.application.*
import io.ktor.server.plugins.di.*
import io.t8n.kedeng.TrainPositions
import io.t8n.kedeng.api.graphql.appScalars
import io.t8n.kedeng.api.graphql.journeyConfig
import io.t8n.kedeng.api.graphql.serviceConfig
import io.t8n.kedeng.api.graphql.trainPositionConfig

fun Application.configureGraphQL() {
    val trainPositions: TrainPositions by dependencies

    install(GraphQL) {
        playground = true
        introspection = true

        schema {
            appScalars()
            serviceConfig()
            journeyConfig(trainPositions)
            trainPositionConfig(trainPositions)
        }

    }
}