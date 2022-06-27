#!/bin/sh
rm -rf ./perf
mkdir perf
cargo build --profile release
/home/karpierz/WSL2-Linux-Kernel/tools/perf/perf record -F 2000 --call-graph dwarf -g target/release/local testing_1024 1024 1024
/home/karpierz/WSL2-Linux-Kernel/tools/perf/perf report  -g graph --header --no-children >> perf.txt
rustfilt -i perf.txt -o demangled.txt
mv demangled.txt ./perf/demangled.txt
rm perf.txt