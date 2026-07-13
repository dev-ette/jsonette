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

package bridge;

public class Main {
    public static void main(String[] args) {
        System.out.println("Calling Rust ping from Java...");
        
        // The Kotlin compiler packages top-level functions in a class named <FileName>Kt
        String result = jsonette.JsonetteKt.ping("world");
        System.out.println("Result: " + result);

        if ("pong: world".equals(result)) {
            System.out.println("UniFFI Java bridge OK");
        } else {
            System.err.println("Bridge failed: unexpected result " + result);
            System.exit(1);
        }
    }
}
