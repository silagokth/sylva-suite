#!/bin/bash

REPO_PATH=$(pwd)
echo "Repo path: ${REPO_PATH}"

logfile="${REPO_PATH}/logs/running.log"
errfile="${REPO_PATH}/logs/error.log"

mkdir -p "${REPO_PATH}/logs"

# Function to run command with redirected logs
function nullify() {
  "$@" >"${logfile}" 2>"${errfile}"
}

echo "running experiment 1"
for i in 16 32 64 128; do
  echo "parameter (address pattern) = $i" | tee -a logs/Exp1.log
  nullify pushd examples/copy
  nullify bash ./run.sh $i
  nullify popd
  for j in {1..8}; do
    python3 -m tb.testcase -n exp1 -p $i
    nullify bash ./run.sh
    grep "Elapsed time" "${logfile}" | tee -a logs/Exp1.log
  done
done

echo "running experiment 2"
nullify pushd examples/copy
nullify bash ./run.sh 32
nullify popd
for i in 2 4 8 16; do
  echo "number of nodes = $i" | tee -a logs/Exp2.log
  for j in {1..8}; do
    python3 -m tb.testcase -n exp2 -p $i
    nullify bash ./run.sh
    grep "Elapsed time" "${logfile}" | tee -a logs/Exp2.log
  done
done

# default setting: set token to 16
echo "Running Sobel Example."
nullify pushd examples/sobel
nullify bash ./run.sh
nullify popd
python3 -m tb.testcase -n sobel
nullify bash ./run.sh
grep "Elapsed time" "${logfile}" | tee -a logs/sobel.log

echo "Running Sobel LeNet-5."
nullify pushd examples/lenet5
nullify bash ./run.sh
nullify popd
python3 -m tb.testcase -n lenet5
nullify bash ./run.sh
grep "Elapsed time" "${logfile}" | tee -a logs/lenet5.log
