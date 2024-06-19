#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import data_structure_pb2 as ds
import copy
import matplotlib.pyplot as plt
import random
import json

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import MessageToDict
from google.protobuf.json_format import Parse

import logging

def generate_simulation_files(db: ds.DataBase, sim_dir: str):
    components_dir = os.path.join(sim_dir, 'components')
    os.makedirs(components_dir, exist_ok=True)

    # create a json file to describe the app graph
    j = MessageToDict(db.app_graph)
    with open(os.path.join(components_dir, 'app_graph.json'), 'w+') as f:
        json.dump(j, f)

    # create table for fire time to write protobuf object db.synthesized_information.node_fire_times to json
    j= MessageToDict(db.synthesized_information)
    with open(os.path.join(components_dir, 'fire_time.json'), 'w+') as f:
        json.dump(j['nodeFireTimes'],f)

    # create a folder for each node
    for node in db.app_graph.nodes:
        node_dir = os.path.join(components_dir, node.id)
        os.makedirs(node_dir, exist_ok=True)
    
    # create a folder for each edge
    for edge in db.app_graph.edges:
        edge_dir = os.path.join(components_dir, edge.id)
        os.makedirs(edge_dir, exist_ok=True)
    
    # write input and output address pattern to each component folder
    for binding in db.synthesized_information.alimp_bindings:
        input_pattern_file = os.path.join(components_dir, binding.app_node_id, 'input_address_pattern.json')
        output_pattern_file = os.path.join(components_dir, binding.app_node_id, 'output_address_pattern.json')
        instance = binding.alimp_instance
        if len(instance.input_addr_time_patterns) >0:
            input_pattern = instance.input_addr_time_patterns
            input_pattern = [{'address': x.key, 'time': x.value} for x in input_pattern]
            with open(input_pattern_file, 'w+') as f:
                json.dump(input_pattern, f)
        if len(instance.output_addr_time_patterns) >0:
            output_pattern = instance.output_addr_time_patterns
            output_pattern = [{'address': x.key, 'time': x.value} for x in output_pattern]
            with open(output_pattern_file, 'w+') as f:
                json.dump(output_pattern, f)
    
    # write address translation table of each port to each component folder
    for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
        port = chunk_address_assignment.port_id
        chunk_address_assignment_file = os.path.join(components_dir, chunk_address_assignment.app_node_id, 'address_translation_table_'+port+'.json')
        for x in chunk_address_assignment.address_assignment:
            print(x)
        att = [{'virtual_address': x, 'physical_address': chunk_address_assignment.address_assignment[x]} for x in chunk_address_assignment.address_assignment]
        with open(chunk_address_assignment_file, 'w+') as f:
            json.dump(att, f)
    
    # write transport table of each edge to each edge folder
    for edge in db.app_graph.edges:
        edge_file = os.path.join(components_dir, edge.id, 'address_transport_table.json')
        tt = db.synthesized_information.transport_tables[edge.id].entries
        att = [{'time': x.time, 'source_address': x.source_address, 'target_address': x.target_address} for x in tt]
        with open(edge_file, 'w+') as f:
            json.dump(att, f)
    
    # write the delay for each edge to each edge folder
    for edge in db.app_graph.edges:
        edge_file = os.path.join(components_dir, edge.id, 'delay.json')
        delay = -1
        for rp in db.synthesized_information.routing_paths:
            if rp.app_edge_id == edge.id:
                delay = rp.delay
                break
        if delay < 0:
            logging.error('No delay found for edge %s', edge.id)
            sys.exit(1)
        with open(edge_file, 'w+') as f:
            json.dump({"delay" : delay}, f)


def run_simulation(sim_dir: str):
    pass

def verify_simulation(sim_dir: str) -> bool:
    return True

def run(db: ds.DataBase, output_dir: str) -> bool:
    logging.info("Start: ideal simulation")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    sim_dir = os.path.join(output_dir, 'ideal_sim')
    os.makedirs(sim_dir, exist_ok=True)
    generate_simulation_files(db, sim_dir)
    run_simulation(sim_dir)
    result = verify_simulation(sim_dir)
    logging.info("Finish: ideal simulation")
    return result
