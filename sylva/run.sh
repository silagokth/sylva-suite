#!/bin/sh

if [ ! -e "./.venv/" ]; then
  echo "Error: Python environment does not exist."
  exit 1
fi

# run the application with the provided parameters
.venv/bin/python3 -m src.main $@
