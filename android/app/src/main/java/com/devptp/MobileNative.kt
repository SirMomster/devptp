package com.devptp

internal object MobileNative {
    init { System.loadLibrary("devptp") }

    external fun start(): Long
    external fun snapshot(handle: Long): String
    external fun connect(handle: Long, ticket: String): String
    external fun disconnect(handle: Long): String
    external fun ping(handle: Long): String
    external fun shutdown(handle: Long)
}
