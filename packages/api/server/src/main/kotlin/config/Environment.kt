package io.t8n.kedeng.config

object Environment {
    fun get(name: String, default: String? = null): String? = System.getenv(name) ?: default
    fun get(name: String): String =
        System.getenv(name) ?: throw RuntimeException("Environment variable $name is required")
}