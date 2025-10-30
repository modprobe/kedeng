package io.t8n.kedeng.api.dto

import io.t8n.kedeng.database.entity.Service as ServiceEntity
import kotlinx.serialization.Serializable
import kotlin.uuid.Uuid
import kotlin.uuid.toKotlinUuid

@Serializable
data class Service(
    val id: Uuid,
    val provider: String,
    val trainType: String,
    val trainNumber: String,
    val timetableYear: Int,
) {
    companion object {
        fun fromEntity(service: ServiceEntity) = Service(
            service.id.value.toKotlinUuid(),
            service.provider,
            service.trainType,
            service.trainNumber,
            service.timetableYear,
        )
    }
}

fun ServiceEntity.toView() = Service.fromEntity(this)