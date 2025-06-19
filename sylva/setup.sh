#!/bin/bash

function nullify() {
  "$@" >/dev/null 2>&1
}

# install a Python environment and its required dependencies
# apt install python3
# apt install python3.12-venv
# apt install protoc
echo "Making Python environment..."
rm -rf .venv/
mkdir .venv/ 
python3 -m venv .venv/
.venv/bin/pip install google numpy matplotlib graphviz
.venv/bin/pip install "protobuf==5.29.4"
.venv/bin/pip install "ortools==9.12.4544"
.venv/bin/pip install "opencv-python"
.venv/bin/pip install "pyinstaller"
echo "Completed"

# compile data structures
echo "Compiling data structure dependencies..."
protoc --python_out=. ./lib/proto/data_structure.proto
protoc --python_out=. ./lib/glic_sim/proto/*.proto
echo "Completed"

# install dependencies and compile examples
echo "Installing cJSON..."
sudo apt-get install libcjson-dev  
echo "Completed"
echo "Compiling examples..."
nullify pushd examples/copy 
nullify bash ./run.sh 
nullify popd

nullify pushd examples/minimum 
nullify bash ./run.sh 
nullify popd

nullify pushd examples/sobel 
nullify bash ./run.sh 
nullify popd





echo "Completed"
