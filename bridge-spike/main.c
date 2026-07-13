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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "jsonetteFFI.h"

// Helper to convert C string to RustBuffer (UniFFI standard layout)
RustBuffer make_rust_buffer(const char *str) {
    RustBuffer buf;
    buf.len = strlen(str);
    buf.capacity = buf.len;
    buf.data = (uint8_t *)malloc(buf.len);
    memcpy(buf.data, str, buf.len);
    return buf;
}

int main() {
    printf("Calling Rust ping from C...\n");

    RustCallStatus status;
    status.code = 0;

    // Call FFI ping function
    RustBuffer input = make_rust_buffer("world");
    RustBuffer output = uniffi_jsonette_fn_func_ping(input, &status);

    if (status.code != 0) {
        printf("FFI call failed with code %d\n", status.code);
        return 1;
    }

    // Convert output to standard C-string
    char *result = (char *)malloc(output.len + 1);
    memcpy(result, output.data, output.len);
    result[output.len] = '\0';
    printf("Result: %s\n", result);

    int success = (strcmp(result, "pong: world") == 0);
    free(result);

    // Free returned output buffer (Rust took ownership and freed the input buffer)
    RustCallStatus free_status;
    free_status.code = 0;
    ffi_jsonette_rustbuffer_free(output, &free_status);

    if (success) {
        printf("UniFFI C bridge OK\n");
        return 0;
    } else {
        printf("Bridge failed: unexpected result\n");
        return 1;
    }
}
