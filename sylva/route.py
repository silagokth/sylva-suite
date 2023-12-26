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


def dijkstra(routing_graph, source_id: str, target_id: str) -> list:
    # use dijkstra algorithm to find a path from source to target
    # return a list of node ids
    source = None
    target = None
    for node in routing_graph.nodes:
        if node.id == source_id:
            source = node
        elif node.id == target_id:
            target = node

    if source is None or target is None:
        print("Error: source or target does not exist!")
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


def plot(routing_graph):
    # create figure
    fig = plt.figure()
    ax = fig.add_subplot(111)

    # plot grid with light grey color
    max_x = 10
    max_y = 10
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

    # show plot
    plt.show()


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
