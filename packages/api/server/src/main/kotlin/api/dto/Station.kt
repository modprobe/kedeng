package io.t8n.kedeng.api.dto

import io.t8n.kedeng.database.entity.Station as StationEntity
import kotlinx.serialization.Serializable

@Serializable
data class StationCodes(
    val ns: String,
    val uic: String,
    val uicCd: String,
    val eva: String,
    val cd: String,
)

@Serializable
data class StationNames(
    val long: String,
    val medium: String,
    val short: String,
)

@Serializable
data class Point(
    val lat: Double,
    val lon: Double,
)

@Serializable
data class Station(
    val codes: StationCodes,
    val names: StationNames,
    val location: Point,
) {
    companion object {
        fun fromEntity(station: StationEntity) = Station(
            StationCodes(
                station.code.value,
                station.uicCode,
                station.uicCdCode,
                station.evaCode,
                station.cdCode,
            ),
            StationNames(
                station.nameLong,
                station.nameMedium,
                station.nameShort,
            ),
            Point(station.location.x, station.location.y)
        )
    }
}

fun StationEntity.toView() = Station.fromEntity(this)