#!/bin/bash

function nullify() {
  "$@" >/dev/null 2>&1
}


echo "running experiment 1"
for i in 16 32 64 128
do
    echo "parameter (address pattern) = $i"
    nullify pushd examples/copy
    nullify bash ./run.sh $i
    nullify popd
    python3 -m tb.testcase -n exp1 -p $i
    ./run.sh 2>/dev/null | grep "Elapsed time"
done

echo ""
echo "running experiment 2"
nullify pushd examples/copy
nullify bash ./run.sh 32
nullify popd
for i in 2 4 8 16
do
    echo "number of nodes = $i"
    python3 -m tb.testcase -n exp2 -p $i
    ./run.sh 2>/dev/null | grep "Elapsed time"
done

# default setting: set token to 16 
nullify pushd examples/copy
nullify bash ./run.sh 16
nullify popd 
