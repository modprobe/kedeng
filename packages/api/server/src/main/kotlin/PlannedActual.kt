package io.t8n.kedeng

import kotlinx.datetime.LocalTime
import kotlinx.serialization.Serializable

@Serializable
data class PlannedActualTime(
    val planned: LocalTime? = null,
    val actual: LocalTime? = null,
) {
    fun isNull() = planned == null && actual == null
    fun collapseToNull() = if (isNull()) null else this
}

@Serializable
data class PlannedActualString(
    val planned: String? = null,
    val actual: String? = null,
) {
    fun isNull() = planned == null && actual == null
    fun collapseToNull() = if (isNull()) null else this
}