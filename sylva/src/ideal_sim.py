#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import lib.proto.data_structure_pb2 as ds
import copy
import matplotlib.pyplot as plt
import random
import json

import lib.glic_sim.common as common_sim 
import src.glic_sim.sim as sim 

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
        # in_names and out_names are later added in the edge loop
        j['config_map'][node.id] = {
            'is_transporter': False,
            'in_names': [],
            'out_names': [],
            'process_cmd': node.executable
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
        edge_name = "transporter_" + edge.id       
        j['config_map'][edge_name] = {
            'is_transporter': True,
            'in_names': [edge.source_node],
            'out_names': [edge.target_node],
            'delay': delay
        }
        j['config_map'][edge.source_node]['out_names'].append(edge_name)
        j['config_map'][edge.target_node]['in_names'].append(edge_name)
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
    # components_dir = os.path.join(sim_dir, 'components')
    # os.makedirs(components_dir, exist_ok=True)

    # create a json file to describe the app graph
    j = make_config_map(db)
    with open(os.path.join(sim_dir, 'config_map.json'), 'w+') as f:
        json.dump(j, f, indent=2)

    # create table for fire time to write protobuf object db.synthesized_information.node_fire_times to json
    j= make_timetable(db)
    with open(os.path.join(sim_dir, 'time_table.json'), 'w+') as f:
        json.dump(j, f, indent=2)

    # write input and output address pattern to each component folder
    for binding in db.synthesized_information.alimp_bindings:
        input_pattern_file = os.path.join(sim_dir, binding.app_node_id + '_inAP.json')
        output_pattern_file = os.path.join(sim_dir, binding.app_node_id + '_outAP.json')
        instance = binding.alimp_instance
        if len(instance.input_addr_time_patterns) >0:
            j={"addr_ptrn":[]}
            input_pattern = instance.input_addr_time_patterns
            j["addr_ptrn"] = [{'address': str(x.key), 'cycle': str(x.value)} for x in input_pattern]
            with open(input_pattern_file, 'w+') as f:
                json.dump(j, f, indent=2)
        if len(instance.output_addr_time_patterns) >0:
            j={"addr_ptrn":[]}
            output_pattern = instance.output_addr_time_patterns
            j["addr_ptrn"] = [{'address': str(x.key), 'cycle': str(x.value)} for x in output_pattern]
            with open(output_pattern_file, 'w+') as f:
                json.dump(j, f, indent=2)
          
    # write address translation table of each port to each component folder
    # build a dictionary to store the type of each port
    port_type = {}
    for node in db.app_graph.nodes:
        for port in node.input_ports:
            port_type[port.id] = 'in'
        for port in node.output_ports:
            port_type[port.id] = 'out'
    for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
        j={"list":[]}
        port = chunk_address_assignment.port_id
        # check if port is input or output
        port = port_type[port]
        chunk_address_assignment_file = os.path.join(sim_dir, chunk_address_assignment.app_node_id + '_'+port+'TT.json')
        j["list"] = [{'addr_in': x, 'addr_out': chunk_address_assignment.address_assignment[x]} for x in chunk_address_assignment.address_assignment]
        with open(chunk_address_assignment_file, 'w+') as f:
            json.dump(j, f, indent=2)
    
    # write transport table of each edge to each edge folder
    for edge in db.app_graph.edges:
        j={"inst_list":[]}
        edge_file = os.path.join(sim_dir, "transporter_"+ edge.id+ '_TransInst.json')
        tt = db.synthesized_information.transport_tables[edge.id].entries
        j["inst_list"] = [{'cycle': x.time, 'addr_rd': x.source_address, 'addr_wr': x.target_address} for x in tt]
        with open(edge_file, 'w+') as f:
            json.dump(j, f, indent=2)


def run_simulation(sim_dir: str):
    # verbosity level: LOW, MEDIUM, HIGH, FULL, DEBUG, NONE
    verbo = common_sim.verbosity.HIGH
    config_dir = os.path.join(sim_dir, 'config_map.json')
    timetable_dir = os.path.join(sim_dir, 'time_table.json')
    use_json = True
    globalCycle = 0
    top_inst = sim.top(verbo, config_dir, timetable_dir, sim_dir, use_json)
    top_inst.initialise()
    top_inst.doYourThing(globalCycle)
    return None

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
