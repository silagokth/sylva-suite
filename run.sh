#!/bin/bash
./bin/sv-dse --cpu 12 --memory 20 -g config/app_graph.json -c config/global_constraint.json -l config/alimp_lib.json -p config/hyper_parameter.json -t config/technology_constraint.json -o out/
