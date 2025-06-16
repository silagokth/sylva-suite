#!/bin/sh

# install a Python environment and its required dependencies
# apt install python3
# apt install python3.12-venv
# apt install protoc
rm -rf .venv/
mkdir .venv/ 
python3 -m venv .venv/
.venv/bin/pip install google numpy matplotlib graphviz
.venv/bin/pip install "protobuf==5.29.4"
.venv/bin/pip install "ortools==9.12.4544"

# compile data structures
protoc --python_out=. ./lib/proto/data_structure.proto
protoc --python_out=. ./lib/glic_sim/proto/*.proto

# install dependencies and compile examples
sudo apt-get install libcjson-dev  
pushd examples/copy;
cc A.c -o A
cc B.c -o B
cc C.c -lcjson -o C
popd;
pushd examples/minimum;
cc A.c -o A
cc B.c -lcjson -o B
cc C.c -lcjson -o C
popd;


popd;

