#!/bin/bash
cargo clean && 
cargo build -r && 
python3 -m maturin build -F python_ffi -r 