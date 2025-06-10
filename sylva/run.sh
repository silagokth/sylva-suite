#!/bin/sh

if [ ! -e "./.venv/" ]; then
  echo "Error: Python environment does not exist."
  exit 1
fi

# An example sh script file to run the application
.venv/bin/python3 -m src.main
