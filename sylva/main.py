from google.protobuf.json_format import MessageToJson
import sys
import os

import json
import argparse
import logging

import data_structure_pb2 as ds
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import bind
import route

def main(graph_file, constraint_file, output_dir):
    # db = ds.DataBase()

    # # read graph file (json) as protobuf objectw
    # with open(graph_file, 'r') as f:
    #     json_string = f.read()
    #     app_graph = Parse(json_string, ds.AppGraph())
    
    # # read constraint file (json) as protobuf object
    # with open(constraint_file, 'r') as f:
    #     json_string = f.read()
    #     global_constraint = Parse(json_string, ds.GlobalConstraint())
    
    # ds.app_graph = app_graph
    # ds.global_constraint = global_constraint

    # # create output directory if not exist
    # if not os.path.exists(output_dir):
    #     os.makedirs(output_dir)

    # binding
    bind.run()

    
    # # write app graph to json file
    # app_graph_json = MessageToJson(app_graph)
    # with open(os.path.join(output_dir, 'app_graph.json'), 'w') as f:
    #     f.write(app_graph_json)

    # # write global constraint to json file
    # global_constraint_json = MessageToJson(global_constraint)
    # with open(os.path.join(output_dir, 'global_constraint.json'), 'w') as f:
    #     f.write(global_constraint_json)
    


if __name__ == "__main__":
    # create logging object with colors for different levels, starting from DEBUG level
    logging.basicConfig(format='%(levelname)s: %(message)s', level=logging.DEBUG)
    logging.addLevelName(logging.DEBUG, "\033[1;34m%s\033[1;0m" % logging.getLevelName(logging.DEBUG))
    logging.addLevelName(logging.INFO, "\033[1;32m%s\033[1;0m" % logging.getLevelName(logging.INFO))
    logging.addLevelName(logging.WARNING, "\033[1;33m%s\033[1;0m" % logging.getLevelName(logging.WARNING))
    logging.addLevelName(logging.ERROR, "\033[1;31m%s\033[1;0m" % logging.getLevelName(logging.ERROR))
    logging.addLevelName(logging.CRITICAL, "\033[1;41m%s\033[1;0m" % logging.getLevelName(logging.CRITICAL))

    # analyse command line arguments
    # -g: input file for application SDF
    # -c: global constraint file
    # -o: output directory, by default is the current directory "."
    parser = argparse.ArgumentParser()
    parser.add_argument("-g", "--graph", help="SDF graph file")
    parser.add_argument("-c", "--constraint", help="global constraint file")
    parser.add_argument("-o", "--output", help="output directory", default=".")
    args = parser.parse_args()

    if args.graph is None:
        logging.error('No graph file specified')
        sys.exit(1)
    if args.constraint is None:
        logging.error('No constraint file specified')
        sys.exit(1)
    
    # call main function
    main(args.graph, args.constraint, args.output)

    # exit
    sys.exit(0)
