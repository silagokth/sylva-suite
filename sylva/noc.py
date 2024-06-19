#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import data_structure_pb2 as ds
import copy
import matplotlib.pyplot as plt
import random
import json
import re

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import MessageToDict
from google.protobuf.json_format import Parse

import logging

def wire_type_to_assignment(wire_type: str):
    assignment = {'e':None, 'n':None, 'w': None, 's': None, 'reg': False, 'buffer': False}
    # use regex to chop label
    pattern = r'\bwire_?([enws]{2,})?(r)?(b)?\b'
    matches = re.match(pattern, wire_type)
    if matches:
        split_s = [matches.group(1)[i:i+2] for i in range(0, len(matches.group(1)), 2)]
        for s in split_s:
            assignment [s[0]] = s[1]
        if matches.group(2):
            assignment['reg'] = True
        if matches.group(3):
            assignment['buffer'] = True
    return assignment

def assignment_to_wire_type(assignment):
    label = ""
    for direction_in in ['e', 'n', 'w', 's']:
        if assignment[direction_in]:
            label += direction_in + assignment[direction_in]
    if assignment['reg']:
        label += 'r'
    if assignment['buffer']:
        label += 'b'
    if label != "":
        label = "wire_" + label
    else:
        label = "wire"
    return label

def noc_block_buffer_insertion(db: ds.DataBase):
    noc_block_assignment = {}
    for label in db.synthesized_information.wire_assignment:
        assignment = db.synthesized_information.wire_assignment[label]
        noc_block_assignment[label] = wire_type_to_assignment(assignment)
    for routing_path in db.synthesized_information.routing_paths:
        for node in routing_path.path:
            label = "{}_{}".format(node.x, node.y)
            if label not in db.synthesized_information.wire_assignment:
                logging.error("Block not found in noc_block_assignment")
                exit(-1)
            assignment =  noc_block_assignment[label]
            if (assignment['n'] == 's' and not assignment['s'] and not assignment['w'] and not assignment['e']) or (
                assignment['s'] == 'n' and not assignment['n'] and not assignment['w'] and not assignment['e']) or (
                assignment['e'] == 'w' and not assignment['s'] and not assignment['w'] and not assignment['n']) or (
                assignment['w'] == 'e' and not assignment['s'] and not assignment['n'] and not assignment['e']):
                if not assignment['reg']:
                    # 20% chance to insert a buffer
                    if random.random() < 0.2:
                        assignment['buffer'] = True
    for node, assignment in noc_block_assignment.items():
        db.synthesized_information.wire_assignment[node] = assignment_to_wire_type(assignment)

def noc_block_register_insertion(db: ds.DataBase):
    noc_block_assignment = {}
    for label in db.synthesized_information.wire_assignment:
        assignment = db.synthesized_information.wire_assignment[label]
        noc_block_assignment[label] = wire_type_to_assignment(assignment)
    for routing_path in db.synthesized_information.routing_paths:
        # find a random delay number smaller than half of the length of the path
        delay = 1
        for node in routing_path.path:
            if delay == 0:
                break
            label = "{}_{}".format(node.x, node.y)
            if label not in db.synthesized_information.wire_assignment:
                logging.error("Block not found in noc_block_assignment")
                exit(-1)
            assignment =  noc_block_assignment[label]
            if (assignment['n'] == 's' and not assignment['s'] and not assignment['w'] and not assignment['e']) or (
                assignment['s'] == 'n' and not assignment['n'] and not assignment['w'] and not assignment['e']) or (
                assignment['e'] == 'w' and not assignment['s'] and not assignment['w'] and not assignment['n']) or (
                assignment['w'] == 'e' and not assignment['s'] and not assignment['n'] and not assignment['e']):
                # 20% chance to insert a register
                if random.random() < 0.2:
                    assignment['reg'] = True
                    delay += 1
        routing_path.delay = delay
    for node, assignment in noc_block_assignment.items():
        db.synthesized_information.wire_assignment[node] = assignment_to_wire_type(assignment)

def noc_block_identification(db: ds.DataBase):
    # make a dict with all path nodes
    noc_block_assignment = {}
    for routing_path in db.synthesized_information.routing_paths:
        for node in routing_path.path:
            label = "{}_{}".format(node.x, node.y)
            if label not in noc_block_assignment:
                noc_block_assignment[label] = {'e':None, 'n':None, 'w': None, 's': None, 'reg': False, 'buffer': False}
        
        for i in range(len(routing_path.path)):
            coord = routing_path.path[i]
            if i==0:
                coord_prev = ds.Coordinate(x=coord.x, y=coord.y-1)
            else:
                coord_prev = routing_path.path[i-1]
            if i == len(routing_path.path)-1:
                coord_next = ds.Coordinate(x=coord.x, y=coord.y+1)
            else:
                coord_next = routing_path.path[i+1]
            
            if coord_prev.x == coord.x and coord_prev.y == coord.y-1:
                direction_in = 's'
            elif coord_prev.x == coord.x and coord_prev.y == coord.y+1:
                direction_in = 'n'
            elif coord_prev.x == coord.x-1 and coord_prev.y == coord.y:
                direction_in = 'w'
            elif coord_prev.x == coord.x+1 and coord_prev.y == coord.y:
                direction_in = 'e'
            else:
                logging.error('Path is not continuos!')
                exit(-1)
            if coord_next.x == coord.x and coord_next.y == coord.y-1:
                direction_out = 's'
            elif coord_next.x == coord.x and coord_next.y == coord.y+1:
                direction_out = 'n'
            elif coord_next.x == coord.x-1 and coord_next.y == coord.y:
                direction_out = 'w'
            elif coord_next.x == coord.x+1 and coord_next.y == coord.y:
                direction_out = 'e'
            else:
                logging.error('Path is not continuos!')
                exit(-1)
            
            if direction_in == direction_out:
                logging.error("Self loop path detected!")
                exit(-1)
            
            old_assignment = noc_block_assignment["{}_{}".format(coord.x, coord.y)]
            # check compatibility
            if old_assignment[direction_in] != None:
                logging.error("conflict detected at block ({}, {}): ".format(coord.x, coord.y) )
                exit(-1)
            for dir in ['e', 'n', 'w', 's']:
                if old_assignment[dir] == direction_out:
                    logging.error("conflict detected at block ({}, {}): ".format(coord.x, coord.y) )
                    exit(-1)
            
            noc_block_assignment["{}_{}".format(coord.x, coord.y)][direction_in] = direction_out  

    for node, assignment in noc_block_assignment.items():
        db.synthesized_information.wire_assignment[node] = assignment_to_wire_type(assignment)           

def generate_picture(db: ds.DataBase, dir: str):
    # create plt
    fig = plt.figure()
    # set aspect to be equal
    plt.gca().set_aspect('equal', adjustable='box')

    # set max_x and max_y to be db.synthesized_information.max_size
    max_x = db.synthesized_information.max_width
    max_y = db.synthesized_information.max_height

    # plot grid with light grey color, the center of each block is the coordinate of the block
    for i in range(max_x+1):
        plt.plot([i-0.5, i-0.5], [-0.5, max_y-0.5], color='lightgrey')
    for i in range(max_y+1):
        plt.plot([-0.5, max_x-0.5], [i-0.5, i-0.5], color='lightgrey')
    
    # plot nodes
    for node in db.app_graph.nodes:
        x = -1
        y = -1
        width = -1
        height = -1
        for placement in db.synthesized_information.placements:
            if placement.app_node_id == node.id:
                x = placement.x
                y = placement.y
                break
        for binding in db.synthesized_information.alimp_bindings:
            if binding.app_node_id == node.id:
                width = binding.alimp_instance.width
                height = binding.alimp_instance.height
                break
        if x == -1 or y == -1 or width == -1 or height == -1:
            logging.error("Error: cannot find placement or binding for node %s" % node.id)
            sys.exit(1)
        
        # draw a rectangle with orange color to represent the node
        plt.gca().add_patch(plt.Rectangle((x-0.5, y-0.5), width, height, color='orange'))
        # add label to the center of the rectangle
        plt.text(x+1, y+1, node.id, horizontalalignment='center', verticalalignment='center')

        # draw a rectangle with red color to represent the output buffer, the buffer coordinate is +0 offset to the north of the node
        plt.gca().add_patch(plt.Rectangle((x-0.5, y+height-0.5), width, 1, color='red'))

        # draw a rectangle with purple color to represent the input buffer, the buffer coordinate is -1 offset to the south of the node
        plt.gca().add_patch(plt.Rectangle((x-0.5, y-1-0.5), width, 1, color='purple'))

        # draw a rectangle with green color to represent the data transporter, the transporter coordinate is +1 offset to the north of the node
        plt.gca().add_patch(plt.Rectangle((x-0.5, y+height+1-0.5), width, 1, color='green'))



    # plot routing path
    for path in db.synthesized_information.routing_paths:
        for i in range(len(path.path)):
            # plot a block with width=1 and height=1 to represent a path node
            x = path.path[i].x
            y = path.path[i].y
            # draw a rectangle with blue color
            plt.gca().add_patch(plt.Rectangle((x-0.5, y-0.5), 1, 1, color='lightblue'))

            assignment = wire_type_to_assignment(db.synthesized_information.wire_assignment["{}_{}".format(x, y)])

            if assignment['reg']:
                # draw a diagonal line to represent the register
                plt.plot([x-0.4, x+0.4], [y-0.4, y+0.4], color='red')
                
            if assignment['buffer']:
                # draw a red dot at the top left corner to represent the buffer
                plt.plot(x-0.3, y+0.3, 'ro', markersize=1)

            if assignment['n'] == 'e':
                # draw an arrow from north to east
                plt.arrow(x, y+0.5, 0.3, -0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['n'] == 'w':
                # draw an arrow from north to west
                plt.arrow(x, y+0.5, -0.3, -0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['n'] == 's':
                # draw an arrow from north to south
                plt.arrow(x, y+0.5, 0, -0.5, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['s'] == 'e':
                # draw an arrow from south to east
                plt.arrow(x, y-0.5, 0.3, 0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['s'] == 'w':
                # draw an arrow from south to west
                plt.arrow(x, y-0.5, -0.3, 0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['s'] == 'n':
                # draw an arrow from south to north
                plt.arrow(x, y-0.5, 0, 0.5, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['e'] == 'w':
                # draw an arrow from east to west
                plt.arrow(x+0.5, y, -0.5, 0, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['e'] == 's':
                # draw an arrow from east to south
                plt.arrow(x+0.5, y, -0.3, -0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['e'] == 'n':
                # draw an arrow from east to north
                plt.arrow(x+0.5, y, -0.3, 0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['w'] == 's':
                # draw an arrow from west to south
                plt.arrow(x-0.5, y, 0.3, -0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['w'] == 'n':
                # draw an arrow from west to north
                plt.arrow(x-0.5, y, 0.3, 0.3, head_width=0.1, head_length=0.1, fc='k', ec='k')
            if assignment['w'] == 'e':
                # draw an arrow from west to east
                plt.arrow(x-0.5, y, 0.5, 0, head_width=0.1, head_length=0.1, fc='k', ec='k')
    
    # save to pdf file
    plt.savefig(os.path.join(dir, 'layout_graph.pdf'))

def run(db: ds.DataBase, output_dir: str) -> bool:
    logging.info("Start: NoC synthesis")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    module_dir = os.path.join(output_dir, 'noc')
    os.makedirs(module_dir, exist_ok=True)
    noc_block_identification(db)
    noc_block_register_insertion(db)
    noc_block_buffer_insertion(db)
    generate_picture(db, module_dir)
    logging.info("Finish: NoC synthesis")
    return True