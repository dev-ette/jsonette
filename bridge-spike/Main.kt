/*
 * Copyright (c) 2026 DevEtte.
 *
 * This project is dual-licensed under both the MIT License and the
 * Apache License, Version 2.0 (the "License"). You may not use this
 * file except in compliance with one of these licenses.
 *
 * You may obtain a copy of the Licenses at:
 * - MIT: https://opensource.org
 * - Apache 2.0: http://apache.org
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the licenses.
 */

package bridge

import jsonette.ping
import kotlin.system.exitProcess

fun main() {
    println("Calling Rust ping from Kotlin...")
    val result = ping("world")
    println("Result: $result")

    if (result == "pong: world") {
        println("UniFFI Kotlin bridge OK")
    } else {
        println("Bridge failed: unexpected result $result")
        exitProcess(1)
    }
}
