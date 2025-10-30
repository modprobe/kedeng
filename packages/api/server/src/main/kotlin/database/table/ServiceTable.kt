package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.dao.id.UUIDTable

object ServiceTable : UUIDTable("service") {
    val trainType = text("type")
    val provider = text("provider")
    val trainNumber = text("train_number")

    val timetableYear = text("timetable_year").transform({ it.toInt() }, { it.toString() })
}
