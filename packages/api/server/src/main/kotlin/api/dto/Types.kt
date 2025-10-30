package io.t8n.kedeng.api.dto

import kotlinx.datetime.serializers.LocalTimeIso8601Serializer
import kotlinx.serialization.Serializable
import kotlinx.datetime.LocalTime as KLocalTime

typealias LocalTime = @Serializable(LocalTimeIso8601Serializer::class) KLocalTime
