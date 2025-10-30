package io.t8n.kedeng.api.dto

import io.t8n.kedeng.database.entity.RollingStock as RollingStockEntity
import kotlinx.serialization.Serializable

@Serializable
data class RollingStock(
    val departureOrder: Int,
    val materialType: String,
    val materialSubtype: String,
    val materialNumber: String,
) {
    companion object {
        fun fromEntity(rollingStock: RollingStockEntity) = RollingStock(
            rollingStock.departureOrder.value,
            rollingStock.materialType,
            rollingStock.materialSubtype,
            rollingStock.materialNumber,
        )
    }
}

fun RollingStockEntity.toView() = RollingStock.fromEntity(this)