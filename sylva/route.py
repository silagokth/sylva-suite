#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import data_structure_pb2 as ds
import copy
import matplotlib.pyplot as plt
import random

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import logging

def create_routing_graph(db: ds.DataBase) -> ds.RoutingGraph:
    routing_graph = create_fully_connected_routing_graph(db.synthesized_information.max_size, db.synthesized_information.max_size)
    node_map = {}
    for node in db.app_graph.nodes:
        x = -1
        y = -1
        width = -1
        height = -1
        print(db.synthesized_information.placements)
        for placement in db.synthesized_information.placements:
            if placement.app_node_id == node.id:
                x = placement.x
                y = placement.y-1
                break
        for binding in db.synthesized_information.alimp_bindings:
            if binding.app_node_id == node.id:
                width = binding.alimp_instance.width
                height = binding.alimp_instance.height+3
                break
        if x == -1 or y == -1 or width == -1 or height == -1:
            print("Error: cannot find placement or binding for node %s" % node.id)
            sys.exit(1)
        node_map[node.id] = (x, y, width, height)
        exclude_nodes = []
        for i in range(len(node.input_ports)):
            exclude_nodes.append("s_" + str(i))
        for i in range(len(node.output_ports)):
            exclude_nodes.append("n_" + str(i))
        add_obstacle(routing_graph, x, y, width, height, exclude_nodes)

    for edge in db.app_graph.edges:
        source_node = edge.source_node
        target_node = edge.target_node
        source_port = edge.source_port
        target_port = edge.target_port
        source_x, source_y, source_width, source_height = node_map[source_node]
        target_x, target_y, target_width, target_height = node_map[target_node]
        source_port_idx = -1
        target_port_idx = -1
        for i in range(len(db.app_graph.nodes)):
            if db.app_graph.nodes[i].id == source_node:
                for j in range(len(db.app_graph.nodes[i].output_ports)):
                    if db.app_graph.nodes[i].output_ports[j].id == source_port:
                        source_port_idx = j
                        break
                break
        for i in range(len(db.app_graph.nodes)):
            if db.app_graph.nodes[i].id == target_node:
                for j in range(len(db.app_graph.nodes[i].input_ports)):
                    if db.app_graph.nodes[i].input_ports[j].id == target_port:
                        target_port_idx = j
                        break
                break
        source_port = "n_" + str(source_port_idx)
        target_port = "s_" + str(target_port_idx)
        source = block_port_id_to_node_id(source_x, source_y, source_width, source_height, source_port)
        target = block_port_id_to_node_id(target_x, target_y, target_width, target_height, target_port)
        routing_graph.channels.add(app_edge_id = edge.id, source=source, target=target, traffic=edge.token_size)
    
    return routing_graph


def load_routing_graph_from_bin(file_name) -> ds.RoutingGraph:
    with open(file_name, 'rb') as f:
        routing_graph = ds.RoutingGraph()
        routing_graph.ParseFromString(f.read())
        return routing_graph


def load_routing_graph_from_json(file_name) -> ds.RoutingGraph:
    with open(file_name, 'r') as f:
        json_string = f.read()
        return Parse(json_string, ds.RoutingGraph())


def load_routing_graph(file_name) -> ds.RoutingGraph:
    if not os.path.isfile(file_name):
        print("File %s does not exist!" % file_name)
        sys.exit(1)

    if file_name.endswith('.json'):
        # if file is a json file
        return load_routing_graph_from_json(file_name)
    else:
        # if file is a binary file
        return load_routing_graph_from_bin(file_name)


def write_routing_graph_to_bin(routing_graph, file_name):
    with open(file_name, 'wb') as f:
        f.write(routing_graph.SerializeToString())


def write_routing_graph_to_json(routing_graph, file_name):
    with open(file_name, 'w') as f:
        f.write(MessageToJson(routing_graph))


def write_routing_graph(routing_graph, file_name):
    if file_name.endswith('.json'):
        write_routing_graph_to_json(routing_graph, file_name)
    else:
        write_routing_graph_to_bin(routing_graph, file_name)


def manhattan_distance(node1: ds.Node, node2: ds.Node) -> int:
    x1, y1, _ = node1.id.split('_')
    x2, y2, _ = node2.id.split('_')
    return abs(int(x1)-int(x2)) + abs(int(y1)-int(y2))

def test_log():
    logging.debug("debug")
    logging.info("info")
    logging.warning("warning")
    logging.error("error")
    logging.critical("critical")


def dijkstra(routing_graph, source_id: str, target_id: str) -> list:
    # use dijkstra algorithm to find a path from source to target
    # return a list of node ids
    

    source = None
    target = None
    for node in routing_graph.nodes:
        if node.id.startswith("3"):
            print(node.id)
        if node.id == source_id:
            source = node
        elif node.id == target_id:
            target = node
    print(source_id, target_id)
    if source is None or target is None:
        logging.error("Error: source or target does not exist!")
        sys.exit(1)

    distances = {}
    for node in routing_graph.nodes:
        if node.id == source_id:
            distances[node.id] = 0
        else:
            distances[node.id] = float('inf')

    visited = set()
    path = []
    path_map = {}

    for count in range(len(routing_graph.nodes)):
        # find node with minimum distance from source
        min_node_id = None
        for node_id in distances:
            if node_id not in visited:
                if min_node_id is None:
                    min_node_id = node_id
                elif distances[node_id] < distances[min_node_id]:
                    min_node_id = node_id
        if min_node_id is None:
            print("Error: cannot find a path from %s to %s" %
                  (source_id, target_id))
            sys.exit(1)
        visited.add(min_node_id)

        # if min_node is target, return path
        if min_node_id == target_id:
            path.append(min_node_id)
            while min_node_id != source_id:
                min_node_id = path_map[min_node_id]
                path.append(min_node_id)
            return path[::-1]

        # for each neighbor of min_node
        for edge in routing_graph.edges:
            if edge.source == min_node_id:
                neighbor_id = edge.target
            elif edge.target == min_node_id:
                neighbor_id = edge.source
            else:
                continue
            # if neighbor is not in visited, update its distance
            if neighbor_id not in visited:
                print(distances[min_node_id],
                      edge.weight, distances[neighbor_id])
                if distances[min_node_id] + edge.weight < distances[neighbor_id]:
                    distances[neighbor_id] = distances[min_node_id] + \
                        edge.weight
                    path_map[neighbor_id] = min_node_id
                    print(path_map)


def a_star(routing_graph, source_id: str, target_id: str) -> list:
    weight_map = {}
    distance_from_start_map = {}

    for node in routing_graph.nodes:
        if node.id == source_id:
            source = node
        elif node.id == target_id:
            target = node

    if source is None or target is None:
        print("Error: source or target does not exist!")
        sys.exit(1)

    # step 1: initialize, set weight of all nodes to its manhattan distance to target
    for node in routing_graph.nodes:
        weight_map[node.id] = manhattan_distance(node, target)
        if node.id == source_id:
            distance_from_start_map[node.id] = 0
        else:
            distance_from_start_map[node.id] = float('inf')

    # step 2: initialize open and closed list
    open_list = [source.id]
    closed_list = []
    path = []
    parent_map = {}
    neighbor_map = {}

    # find all neighbors for each node
    for node in routing_graph.nodes:
        neighbor_map[node.id] = []
        for edge in routing_graph.edges:
            if edge.source == node.id:
                neighbor_map[node.id].append(edge.target)
            elif edge.target == node.id:
                neighbor_map[node.id].append(edge.source)

    print(neighbor_map)

    # step 3: loop until open list is empty
    while open_list:

        # step 4: find node with minimum weight in open list
        min_node_id = open_list[0]
        for node_id in open_list:
            if weight_map[node_id] < weight_map[min_node_id]:
                min_node_id = node_id
        print("choose %s" % min_node_id)
        # step 5: remove min_node from open list and add it to closed list
        open_list.remove(min_node_id)
        closed_list.append(min_node_id)

        # step 6: if min_node is target, return path
        if min_node_id == target.id:
            print(parent_map)
            path.append(min_node_id)
            while parent_map[min_node_id] != source.id:
                path.append(parent_map[min_node_id])
                min_node_id = parent_map[min_node_id]
            path.append(source.id)
            return path[::-1]

        # step 7: for each neighbor of min_node
        for neighbor_id in neighbor_map[min_node_id]:
            # step 8: if neighbor is in closed list, skip
            if neighbor_id in closed_list:
                continue
            # step 9: if neighbor is not in open list, add it to open list
            if neighbor_id not in open_list:
                open_list.append(neighbor_id)
            # step 10: calculate new weight for neighbor
            edge_min_node_neighbor = None
            for edge in routing_graph.edges:
                if edge.source == min_node_id and edge.target == neighbor_id:
                    edge_min_node_neighbor = edge
                    break
                elif edge.source == neighbor_id and edge.target == min_node_id:
                    edge_min_node_neighbor = edge
                    break
            if edge_min_node_neighbor is None:
                print("Error: edge between %s and %s does not exist!" %
                      (min_node_id, neighbor_id))
                sys.exit(1)

            if distance_from_start_map[min_node_id] + edge_min_node_neighbor.weight < distance_from_start_map[neighbor_id]:
                parent_map[neighbor_id] = min_node_id
                distance_from_start_map[neighbor_id] = distance_from_start_map[min_node_id] + \
                    edge_min_node_neighbor.weight

            new_weight = distance_from_start_map[neighbor_id] + \
                weight_map[neighbor_id]
            print(new_weight, weight_map[min_node_id], weight_map)
            # step 11: if new weight is smaller than neighbor's weight, update neighbor's weight and parent
            if new_weight < weight_map[min_node_id]:
                weight_map[neighbor_id] = new_weight
                parent_map[neighbor_id] = min_node_id
                print("update %s" % neighbor_id)

    # step 12: if open list is empty, return empty path
    return path


def find_connected_channel_groups(routing_graph: ds.RoutingGraph) -> list:
    connected_channel_groups = []
    node_map = {}
    for channel in routing_graph.channels:
        if channel.source in node_map:
            i = node_map[channel.source]
            node_map[channel.target] = i
            connected_channel_groups[i].append(channel)
        elif channel.target in node_map:
            i = node_map[channel.target]
            node_map[channel.source] = i
            connected_channel_groups[i].append(channel)
        else:
            i = len(connected_channel_groups)
            node_map[channel.source] = i
            node_map[channel.target] = i
            connected_channel_groups.append([channel])

    # sort connected_channel_groups by their channel traffic per channel, high to low
    connected_channel_groups.sort(key=lambda x: sum(
        [channel.traffic for channel in x])/len(x), reverse=True)
    return connected_channel_groups


def can_channels_share_path(channel1, channel2):
    if channel1.source == channel2.source:
        return True
    if channel1.target == channel2.target:
        return True
    else:
        return False


def remove_nodes(routing_graph, nodes):
    # remove all nodes in notes
    for node_id in nodes:
        for node in routing_graph.nodes:
            if node.id == node_id:
                routing_graph.nodes.remove(node)
                break
    # remove all edges that has one end in nodes
    edges = []
    for edge in routing_graph.edges:
        if edge.source not in nodes and edge.target not in nodes:
            edges.append(edge)
    # clear routing_graph.edges
    routing_graph.edges.clear()
    # add edges back to routing_graph.edges
    routing_graph.edges.extend(edges)


def route(routing_graph):
    # order channels  by their traffic, high to low
    routing_graph.channels.sort(key=lambda x: x.traffic, reverse=True)
    for idx in range(len(routing_graph.channels)):
        # create a deep copy of routing graph
        routing_graph_copy = copy.deepcopy(routing_graph)
        channel = routing_graph.channels[idx]
        for i in range(idx):
            if not can_channels_share_path(channel, routing_graph.channels[i]):
                # remove all nodes in the path of channel
                remove_nodes(routing_graph_copy,
                             routing_graph.channels[i].path)
        # find a path from source to target
        path = dijkstra(routing_graph_copy, channel.source, channel.target)
        routing_graph.channels[idx].path.clear()
        routing_graph.channels[idx].path.extend(path)


def node_id_to_xy(node_id) -> tuple:
    x, y, d = node_id.split('_')
    x = int(x)
    y = int(y)
    d = int(d)
    if d == 2:
        dx = 0.5
        dy = 0.5
    elif d == 0:
        dx = 1
        dy = 0.5
    elif d == 1:
        dx = 0.5
        dy = 1
    return (x+dx, y+dy)


def plot(routing_graph, max_x, max_y):
    # create figure
    fig = plt.figure()
    ax = fig.add_subplot(111)

    # plot grid with light grey color
    for i in range(max_x+1):
        ax.plot([i, i], [0, max_y], color='lightgrey')
    for i in range(max_y+1):
        ax.plot([0, max_x], [i, i], color='lightgrey')

    # plot nodes
    for node in routing_graph.nodes:
        x, y = node_id_to_xy(node.id)
        ax.scatter(x, y, color='red')

    # plot edges in grey color with dashed line
    for edge in routing_graph.edges:
        x1, y1 = node_id_to_xy(edge.source)
        x2, y2 = node_id_to_xy(edge.target)
        ax.plot([x1, x2], [y1, y2], color='grey', linestyle='dashed')

    # plot paths for each channel
    for channel in routing_graph.channels:
        path = channel.path
        random_dx = random.random()*0.1 - 0.05
        random_dy = random.random()*0.1 - 0.05
        random_color = (random.random(), random.random(), random.random())
        for i in range(len(path)-1):
            x1, y1 = node_id_to_xy(path[i])
            x2, y2 = node_id_to_xy(path[i+1])
            ax.plot([x1+random_dx, x2+random_dx],
                    [y1+random_dy, y2+random_dy], color='blue', linewidth=2)

    # save to pdf file
    plt.savefig('routing_graph.pdf')


def create_fully_connected_routing_graph(max_x, max_y):
    routing_graph = ds.RoutingGraph()
    for x in range(max_x):
        for y in range(max_y):
            if x < max_x-1:
                routing_graph.nodes.add(id='%d_%d_%d' % (x, y, 0))
            if y < max_y-1:
                routing_graph.nodes.add(id='%d_%d_%d' % (x, y, 1))

    # horizontal connection
    for x in range(max_x-2):
        for y in range(max_y):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x, y, 0), target='%d_%d_%d' % (x+1, y, 0), weight=1)
    # vertical connection
    for x in range(max_x):
        for y in range(max_y-2):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x, y, 1), target='%d_%d_%d' % (x, y+1, 1), weight=1)

    # right down diagonal connection
    for x in range(max_x-1):
        for y in range(max_y-1):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x, y, 1), target='%d_%d_%d' % (x, y, 0), weight=1.2)
    for x in range(max_x-1):
        for y in range(max_y-1):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x, y+1, 0), target='%d_%d_%d' % (x+1, y, 1), weight=1.2)

    # left down diagonal connection
    for x in range(max_x-1):
        for y in range(max_y-1):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x, y+1, 0), target='%d_%d_%d' % (x, y, 1), weight=1.2)
    for x in range(max_x-1):
        for y in range(max_y-1):
            routing_graph.edges.add(source='%d_%d_%d' % (
                x+1, y, 1), target='%d_%d_%d' % (x, y, 0), weight=1.2)

    return routing_graph


def block_port_id_to_node_id(x, y, width, height, port_id):
    print(x, y, width, height, port_id)
    d, i = port_id.split('_')
    i = int(i)
    if d == 'N' or d == 'n':
        return '%d_%d_%d' % (x+i, y+height-1, 1)
    elif d == 'S' or d == 's':
        return '%d_%d_%d' % (x+i, y-1, 1)
    elif d == 'W' or d == 'w':
        return '%d_%d_%d' % (x-2, y+i, 0)
    elif d == 'E' or d == 'e':
        return '%d_%d_%d' % (x+width-1, y+i, 0)
    else:
        print("Error: invalid direction %s" % d)
        sys.exit(1)

def path_segment_to_coord(node_id_0, node_id_1):
    x0, y0, d0 = node_id_0.split('_')
    x1, y1, d1 = node_id_1.split('_')
    x0 = int(x0)
    y0 = int(y0)
    d0 = int(d0)
    x1 = int(x1)
    y1 = int(y1)
    d1 = int(d1)

    # west to east
    if x0 == x1-1 and y0 == y1 and d0 == d1 and d0 == 0:
        return ds.Coordinate(x=x1, y=y1)
    # east to west
    elif x0 == x1+1 and y0 == y1 and d0 == d1 and d0 == 0:
        return ds.Coordinate(x=x0, y=y0)
    # south to north
    elif x0 == x1 and y0 == y1-1 and d0 == d1 and d0 == 1:
        return ds.Coordinate(x=x1, y=y1)
    # north to south
    elif x0 == x1 and y0 == y1+1 and d0 == d1 and d0 == 1:
        return ds.Coordinate(x=x0, y=y0)
    # east to north
    elif x0 == x1 and y0 == y1 and d0 == 0 and d1 == 1:
        return ds.Coordinate(x=x1, y=y1)
    # north to east
    elif x0 == x1 and y0 == y1 and d0 == 1 and d1 == 0:
        return ds.Coordinate(x=x0, y=y0)
    # west to north
    elif x0 == x1-1 and y0 == y1 and d0 == 0 and d1 == 1:
        return ds.Coordinate(x=x1, y=y1)
    # north to west
    elif x0 == x1+1 and y0 == y1 and d0 == 1 and d1 == 0:
        return ds.Coordinate(x=x0, y=y0)
    # east to south
    elif x0 == x1 and y0 == y1+1 and d0 == 0 and d1 == 1:
        return ds.Coordinate(x=x0, y=y0)
    # south to east
    elif x0 == x1 and y0 == y1-1 and d0 == 1 and d1 == 0:
        return ds.Coordinate(x=x1, y=y1)
    # west to south
    elif x0 == x1-1 and y0 == y1+1 and d0 == 0 and d1 == 1:
        return ds.Coordinate(x=x0+1, y=y0)
    # south to west
    elif x0 == x1+1 and y0 == y1-1 and d0 == 1 and d1 == 0:
        return ds.Coordinate(x=x0, y=y0+1)
    else:
        print("Error: invalid path segment %s -> %s" % (node_id_0, node_id_1))
        sys.exit(1)


def add_obstacle(routing_graph, x, y, width, height, exclude_nodes):
    nodes = []
    node_ids = set()
    exclude_ids = set()
    for node in exclude_nodes:
        d, i = node.split('_')
        i = int(i)
        if d == 'N' or d == 'n':
            # north
            exclude_ids.add('%d_%d_%d' % (x+i, y+height-1, 1))
        elif d == 'S' or d == 's':
            # south
            exclude_ids.add('%d_%d_%d' % (x+i, y-1, 1))
        elif d == 'W' or d == 'w':
            # west
            exclude_ids.add('%d_%d_%d' % (x-2, y+i, 0))
        elif d == 'E' or d == 'e':
            # east
            exclude_ids.add('%d_%d_%d' % (x+width-1, y+i, 0))
        else:
            print("Error: invalid direction %s" % d)
            sys.exit(1)

    for node in routing_graph.nodes:
        if node.id in exclude_ids:
            nodes.append(copy.deepcopy(node))
            node_ids.add(node.id)
            continue
        i, j, d = node.id.split('_')
        i = int(i)
        j = int(j)
        d = int(d)
        if (int(i) < x-1 or i >= x+width or j < y-1 or j >= y+height):
            nodes.append(copy.deepcopy(node))
            node_ids.add(node.id)
        elif (i == x-1) and (d != 0):
            nodes.append(copy.deepcopy(node))
            node_ids.add(node.id)
        elif (j == y-1) and (d != 1):
            nodes.append(copy.deepcopy(node))
            node_ids.add(node.id)

    # clear routing_graph.nodes
    routing_graph.nodes.clear()
    # add nodes back to routing_graph.nodes
    routing_graph.nodes.extend(nodes)

    # remove all edges that has one end that is not in routing_graph.nodes
    edges = []
    for edge in routing_graph.edges:
        # if both source and target are in exclude_ids, skip
        if edge.source in exclude_ids and edge.target in exclude_ids:
            continue
        if edge.source in node_ids and edge.target in node_ids:
            edges.append(copy.deepcopy(edge))
    # clear routing_graph.edges
    routing_graph.edges.clear()
    # add edges back to routing_graph.edges
    routing_graph.edges.extend(edges)

def generate_picture(db: ds.DataBase):
    # create plt
    fig = plt.figure()

    # set max_x and max_y to be db.synthesized_information.max_size
    max_x = db.synthesized_information.max_size
    max_y = db.synthesized_information.max_size

    # plot grid with light grey color, the center of each block is the coordinate of the block
    for i in range(max_x+2):
        plt.plot([i-0.5, i-0.5], [-0.5, max_y+0.5], color='lightgrey')
    for i in range(max_y+2):
        plt.plot([-0.5, max_x+0.5], [i-0.5, i-0.5], color='lightgrey')
    
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
            print("Error: cannot find placement or binding for node %s" % node.id)
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
            plt.gca().add_patch(plt.Rectangle((x-0.5, y-0.5), 1, 1, color='blue'))
    
    # save to pdf file
    plt.savefig('routing_graph.pdf')



def update_synthesized_info(db: ds.DataBase, routing_graph: ds.RoutingGraph):
    for channel in routing_graph.channels:
        for edge in db.app_graph.edges:
            if edge.id == channel.app_edge_id:
                db.synthesized_information.routing_paths.append(ds.RoutingPath(app_edge_id=edge.id))
                idx = len(db.synthesized_information.routing_paths)-1
                for x in range(len(channel.path)-1):
                    db.synthesized_information.routing_paths[idx].path.append(path_segment_to_coord(channel.path[x], channel.path[x+1]))
                db.synthesized_information.routing_paths[idx].delay = 1 + len(channel.path) % 5
                break
    print(db.synthesized_information.routing_paths)
                    

def run (db: ds.DataBase):
    routing_graph = create_routing_graph(db)
    route(routing_graph)
    #plot(routing_graph, db.synthesized_information.max_size, db.synthesized_information.max_size)
    update_synthesized_info(db, routing_graph)
    generate_picture(db)

if __name__ == '__main__':
    routing_graph = create_fully_connected_routing_graph(10, 10)
    add_obstacle(routing_graph, 2, 2, 3, 3, ["n_1", "s_0"])
    add_obstacle(routing_graph, 7, 6, 2, 3, ["n_0", "s_0"])
    add_obstacle(routing_graph, 6, 2, 3, 2, ["n_2"])

    routing_graph.channels.add(
        source=block_port_id_to_node_id(2, 2, 3, 3, "s_0"), target=block_port_id_to_node_id(7, 6, 2, 3, "n_0"), traffic=1)
    routing_graph.channels.add(
        source=block_port_id_to_node_id(7, 6, 2, 3, "s_0"), target=block_port_id_to_node_id(6, 2, 3, 2, "n_2"), traffic=1)
    routing_graph.channels.add(
        source=block_port_id_to_node_id(7, 6, 2, 3, "s_0"), target=block_port_id_to_node_id(2, 2, 3, 3, "n_1"), traffic=1)
    route(routing_graph)

    plot(routing_graph)
