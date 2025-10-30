package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.dao.id.IdTable

object StationTable : IdTable<String>("station") {
    override val id = text("code").entityId()
    val uicCode = text("uic_code")
    val uicCdCode = text("uic_cd_code")
    val evaCode = text("eva_code")
    val cdCode = text("cd_code")

    val stationType = text("station_type")

    val nameLong = text("name_long")
    val nameMedium = text("name_medium")
    val nameShort = text("name_short")
    val nameSynonyms = array<String>("name_synonyms")

    val country = varchar("country", 255)
    val tracks = array<String>("tracks")

    val hasTravelAssistance = bool("has_travel_assistance")
    val isBorderStop = bool("is_border_stop")
    val isAvailableForAccessibleTravel = bool("is_available_for_accessible_travel")
    val hasKnownFacilities = bool("has_known_facilities")
    val areTracksIndependentlyAccessible = bool("are_tracks_independently_accessible")

    val location = point("location")
}