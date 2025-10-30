package io.t8n.kedeng.api.graphql

import com.apurebase.kgraphql.schema.dsl.SchemaBuilder
import io.t8n.kedeng.api.dto.toView
import io.t8n.kedeng.database.entity.Service
import org.jetbrains.exposed.v1.jdbc.transactions.transaction
import kotlin.uuid.Uuid
import kotlin.uuid.toJavaUuid

fun SchemaBuilder.serviceConfig() {
    query("services") {
        resolver { -> transaction { Service.all().map(Service::toView) } }
    }

    query("service") {
        resolver { id: Uuid ->
            transaction {
                Service.findById(id.toJavaUuid())?.toView()
            }
        }
    }
}