package io.t8n.kedeng.database.entity

import io.t8n.kedeng.database.table.StationTable
import org.jetbrains.exposed.v1.core.dao.id.EntityID
import org.jetbrains.exposed.v1.dao.Entity
import org.jetbrains.exposed.v1.dao.ImmutableEntityClass

class Station(id: EntityID<String>) : Entity<String>(id) {
    companion object : ImmutableEntityClass<String, Station>(StationTable)

    val code by StationTable.id
    val uicCode by StationTable.uicCode
    val uicCdCode by StationTable.uicCdCode
    val evaCode by StationTable.evaCode
    val cdCode by StationTable.cdCode

    val stationType by StationTable.stationType

    val nameLong by StationTable.nameLong
    val nameMedium by StationTable.nameMedium
    val nameShort by StationTable.nameShort
    val nameSynonyms by StationTable.nameSynonyms

    val location by StationTable.location
}