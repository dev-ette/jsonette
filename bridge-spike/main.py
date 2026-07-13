# Copyright (c) 2026 DevEtte.
#
# This project is dual-licensed under both the MIT License and the
# Apache License, Version 2.0 (the "License"). You may not use this
# file except in compliance with one of these licenses.
#
# You may obtain a copy of the Licenses at:
# - MIT: https://opensource.org
# - Apache 2.0: http://apache.org
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the licenses.

import sys
import os

# Add bridge-spike directory to import search path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

import jsonette

print("Calling Rust ping from Python...")
result = jsonette.ping("world")
print(f"Result: {result}")

if result == "pong: world":
    print("UniFFI Python bridge OK")
else:
    print(f"Unexpected result: {result}")
    sys.exit(1)
