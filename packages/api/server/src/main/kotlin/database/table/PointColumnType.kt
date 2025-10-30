package io.t8n.kedeng.database.table

import org.jetbrains.exposed.v1.core.Column
import org.jetbrains.exposed.v1.core.ColumnType
import org.jetbrains.exposed.v1.core.Table
import org.postgresql.geometric.PGpoint

fun Table.point(name: String): Column<PGpoint> = registerColumn(name, PointColumnType())

private class PointColumnType : ColumnType<PGpoint>() {
    override fun sqlType() = "POINT"

    override fun valueFromDB(value: Any): PGpoint? {
        return value as? PGpoint
    }
}