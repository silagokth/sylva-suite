#!/bin/bash

rm -rf examples/
mkdir examples/

mkdir examples/minimum/
mkdir examples/minimum/mem
cp modules/target/debug/minimum-A examples/minimum/A
cp modules/target/debug/minimum-B examples/minimum/B
cp modules/target/debug/minimum-C examples/minimum/C
./modules/target/debug/minimum-memory --image examples/minimum/mem/global_mem_image.json --reference examples/minimum/mem/global_mem_reference.json

# move binaries to bin
rm -rf bin/
mkdir bin/
cp modules/target/debug/sylva bin/sylva
cp modules/target/debug/sv-sim bin/sv-sim
cp modules/target/debug/config bin/config

