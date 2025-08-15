#!/bin/bash

../../../.venv/bin/python3 input.py input.png global_mem_image.json
cp global_mem_image.json global_mem_reference.json
./load --global-image global_mem_image.json --out-mem _load.json
./gx --in-mem _load.json --out-mem _gx.json
./gy --in-mem _load.json --out-mem _gy.json
python3 combine_json.py _gx.json _gy.json _input_combine.json
./combine --in-mem _input_combine.json --out-mem _output_combine.json
./store --global-image global_mem_reference.json --in-mem _output_combine.json
mv global_mem_image.json ../mem/
mv global_mem_reference.json ../mem/
rm _*
