import sys
import os

import json
import argparse
import logging

import lib.proto.data_structure_pb2 as ds
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import src.bind as bind
import src.place as place
import src.route as route
import src.noc as noc 
import src.glic as glic 
import src.ideal_sim as ideal_sim
import src.mem_syn as mem_syn
import tb.testcase as testcase



def main(graph_file, constraint_file, library_file, parameter_file, noc_file, output_dir):
    db = ds.DataBase()

    # read graph file (json) as protobuf objectw
    with open(graph_file, 'r') as f:
        json_string = f.read()
        Parse(json_string, db.app_graph)
    
    # read constraint file (json) as protobuf object
    with open(constraint_file, 'r') as f:
        json_string = f.read()
        Parse(json_string, db.global_constraint)
    
    # read library file (json) as protobuf object
    with open(library_file, 'r') as f:
        json_string = f.read()
        Parse(json_string, db.alimp_lib)

    # read parameter file (json) as protobuf object
    with open(parameter_file, 'r') as f:
        json_string = f.read()
        Parse(json_string, db.hyper_parameter)

    # read noc file (json) as protobuf object
    with open(noc_file, 'r') as f:
        json_string = f.read()
        Parse(json_string, db.noc_constraint)

    # create output directory if not exist
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # binding
    bind.run(db, output_dir)
    
    
    while(True):
        if len(db.alimp_binding_options) == 0:
            logging.error('No binding option left')
            sys.exit(1)
        
        # place
        del db.synthesized_information.alimp_bindings[:]
        for binding in db.alimp_binding_options[0].alimp_bindings:
            db.synthesized_information.alimp_bindings.append(binding)
        result = place.run(db, output_dir)
        if not result:
            db.alimp_binding_options.pop(0)
            continue

        # route
        route.run(db, output_dir)

        # noc
        result = noc.run(db, output_dir)
        if not result:
            db.alimp_binding_options.pop(0)
            continue

        # glic
        glic.run(db)

        # write protobuf object "db" to a binary file
        with open(os.path.join(output_dir, 'db.bin'), 'wb') as f:
            f.write(db.SerializeToString())

        # read protobuf object "db" from a binary file
        with open(os.path.join(output_dir, 'db.bin'), 'rb') as f:
            db.ParseFromString(f.read())

        # ideal simulation
        result = ideal_sim.run(db, output_dir)
        if not result:
            db.alimp_binding_options.pop(0)
            continue
        
        # memory synthesis
        result = mem_syn.run(db, output_dir)
        if not result:
            db.alimp_binding_options.pop(0)
            continue

        break

    logging.info("Sylva finished successfully!")

    
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
    logging.basicConfig(format='%(levelname)s: %(message)s', level=logging.INFO)
    logging.addLevelName(logging.DEBUG, "\033[1;34m%s\033[1;0m" % logging.getLevelName(logging.DEBUG))
    logging.addLevelName(logging.INFO, "\033[1;32m%s\033[1;0m" % logging.getLevelName(logging.INFO))
    logging.addLevelName(logging.WARNING, "\033[1;33m%s\033[1;0m" % logging.getLevelName(logging.WARNING))
    logging.addLevelName(logging.ERROR, "\033[1;31m%s\033[1;0m" % logging.getLevelName(logging.ERROR))
    logging.addLevelName(logging.CRITICAL, "\033[1;41m%s\033[1;0m" % logging.getLevelName(logging.CRITICAL))

    # analyse command line arguments
    # -g: input file for application SDF
    # -c: global constraint file
    # -l: alimp library file
    # -p: hyper parameter file
    # -n: noc constraint file
    # -o: output directory, by default is the current directory "."
    parser = argparse.ArgumentParser()
    parser.add_argument("-g", "--graph", help="SDF graph file", default="const/app_graph.json")
    parser.add_argument("-c", "--constraint", help="global constraint file", default="const/global_constraint.json")
    parser.add_argument("-l", "--library", help="alimp library file", default="const/alimp_lib.json")
    parser.add_argument("-p", "--parameter", help="hyper parameter file", default="const/hyper_parameter.json")
    parser.add_argument("-n", "--noc", help="noc constraint file", default="const/noc_constraint.json")
    parser.add_argument("-o", "--output", help="output directory", default="bin/")
    args = parser.parse_args()

    # create test database
    testcase.create_test_db("minimum")
    
    # call main function
    main(args.graph, args.constraint, args.library, args.parameter, args.noc, args.output)

    # exit
    sys.exit(0)
