#!/bin/bash
pushd modules/
cargo build --all
popd

rm -rf examples/
mkdir examples/

mkdir -p examples/minimum/mem
cp modules/target/debug/minimum-A examples/minimum/A
cp modules/target/debug/minimum-B examples/minimum/B
cp modules/target/debug/minimum-C examples/minimum/C
./modules/target/debug/minimum-memory --image examples/minimum/mem/global_mem_image.json --reference examples/minimum/mem/global_mem_reference.json

mkdir -p examples/copy/mem
cp modules/target/debug/copy-A examples/copy/A
cp modules/target/debug/copy-B examples/copy/B
cp modules/target/debug/copy-C examples/copy/C
./modules/target/debug/copy-memory --image examples/copy/mem/global_mem_image.json --reference examples/copy/mem/global_mem_reference.json

mkdir -p examples/sobel/mem
cp modules/target/debug/sobel-load examples/sobel/load
cp modules/target/debug/sobel-copy examples/sobel/copy
cp modules/target/debug/sobel-gx examples/sobel/gx
cp modules/target/debug/sobel-gy examples/sobel/gy
cp modules/target/debug/sobel-combine examples/sobel/combine
cp modules/target/debug/sobel-store examples/sobel/store
./modules/target/debug/sobel-memory --image examples/sobel/mem/global_mem_image.json --reference examples/sobel/mem/global_mem_reference.json


# move binaries to bin
rm -rf bin/
mkdir bin/
cp modules/target/debug/sylva bin/sylva
cp modules/target/debug/sv-sim bin/sv-sim
cp modules/target/debug/config bin/config

