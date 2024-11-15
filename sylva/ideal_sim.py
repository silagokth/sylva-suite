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

def make_config_map(db: ds.DataBase) -> dict:
    app_graph = db.app_graph
    j = {
        'config_map':{}
    }
    for node in app_graph.nodes:
        j['config_map'][node.id] = {
            'is_transporter': False,
            'has_input': True if len(node.input_ports) > 0 else False,
            'has_output': True if len(node.output_ports) > 0 else False,
            'process_cmd': 'echo hello!'
        }
    for edge in app_graph.edges:
        delay = -1
        for rp in db.synthesized_information.routing_paths:
            if rp.app_edge_id == edge.id:
                delay = rp.delay
                break
        if delay < 0:
            logging.error('No delay found for edge %s', edge.id)
            sys.exit(1)
        j['config_map']["transporter_"+edge.id] = {
            'is_transporter': True,
            'in_node_name': edge.source_node,
            'out_node_name': edge.target_node,
            'delay': delay
        }
    return j
def make_timetable(db: ds.DataBase) -> dict:
    j = {
        'tt':[]
    }
    synthesized_information = MessageToDict(db.synthesized_information)
    fire_times = synthesized_information['nodeFireTimes']

    for node in fire_times:
        j['tt'].append({
            'cycle': fire_times[node],
            'node_name': node
        })
    return j



def generate_simulation_files(db: ds.DataBase, sim_dir: str):
    components_dir = os.path.join(sim_dir, 'components')
    os.makedirs(components_dir, exist_ok=True)

    # create a json file to describe the app graph
    j = make_config_map(db)
    with open(os.path.join(components_dir, 'config_map.json'), 'w+') as f:
        json.dump(j, f)

    # create table for fire time to write protobuf object db.synthesized_information.node_fire_times to json
    j= make_timetable(db)
    with open(os.path.join(components_dir, 'time_table.json'), 'w+') as f:
        json.dump(j,f)

    # # create a folder for each node
    # for node in db.app_graph.nodes:
    #     node_dir = os.path.join(components_dir, node.id)
    #     os.makedirs(node_dir, exist_ok=True)
    
    # # create a folder for each edge
    # for edge in db.app_graph.edges:
    #     edge_dir = os.path.join(components_dir, edge.id)
    #     os.makedirs(edge_dir, exist_ok=True)
    
    # write input and output address pattern to each component folder
    for binding in db.synthesized_information.alimp_bindings:
        input_pattern_file = os.path.join(components_dir, binding.app_node_id + '_inAP.json')
        output_pattern_file = os.path.join(components_dir, binding.app_node_id + '_outAP.json')
        instance = binding.alimp_instance
        if len(instance.input_addr_time_patterns) >0:
            j={"addr_ptrn":[]}
            input_pattern = instance.input_addr_time_patterns
            j["addr_ptrn"] = [{'address': str(x.key), 'cycle': str(x.value)} for x in input_pattern]
            with open(input_pattern_file, 'w+') as f:
                json.dump(j, f)
        if len(instance.output_addr_time_patterns) >0:
            j={"addr_ptrn":[]}
            output_pattern = instance.output_addr_time_patterns
            j["addr_ptrn"] = [{'address': str(x.key), 'cycle': str(x.value)} for x in output_pattern]
            with open(output_pattern_file, 'w+') as f:
                json.dump(j, f)
    
    # write address translation table of each port to each component folder
    # build a dictionary to store the type of each port
    port_type = {}
    for node in db.app_graph.nodes:
        for port in node.input_ports:
            port_type[port.id] = 'in'
        for port in node.output_ports:
            port_type[port.id] = 'out'
    for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
        port = chunk_address_assignment.port_id
        # check if port is input or output
        port = port_type[port]
        chunk_address_assignment_file = os.path.join(components_dir, chunk_address_assignment.app_node_id + '_'+port+'TT.json')
        for x in chunk_address_assignment.address_assignment:
            print(x)
        att = [{'virtual_address': x, 'physical_address': chunk_address_assignment.address_assignment[x]} for x in chunk_address_assignment.address_assignment]
        with open(chunk_address_assignment_file, 'w+') as f:
            json.dump(att, f)
    
    # write transport table of each edge to each edge folder
    for edge in db.app_graph.edges:
        edge_file = os.path.join(components_dir, "transporter_"+ edge.id+ '_TransInst.json')
        tt = db.synthesized_information.transport_tables[edge.id].entries
        att = [{'cycle': x.time, 'addr_rd': x.source_address, 'addr_wr': x.target_address} for x in tt]
        with open(edge_file, 'w+') as f:
            json.dump(att, f)


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
