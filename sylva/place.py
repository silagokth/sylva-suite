#!/usr/bin/env python3
# -*- coding: utf-8 -*-

## @package place
# This module contains process to place app_graph actors on a 2D plane.
# It uses or-tools to check the feasibility of the placement against the global constraint
# It reports the actual height and width after placement.

import os
import sys
import json
import logging

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import data_structure_pb2 as ds
from ortools.sat.python import cp_model

def create_floor_plan(db: ds.DataBase) -> ds.FloorPlan:
    ''' Create a floor plan with a single square. '''
    fp = ds.FloorPlan()
    fp.max_width = db.global_constraint.max_width
    fp.max_height = db.global_constraint.max_height

    node_map = {}

    # for each actor in app_graph, find the corresponding alimp instance and its width and height, add it to fp
    for node in db.app_graph.nodes:
        for binding in db.synthesized_information.alimp_bindings:
            if binding.app_node_id == node.id:
                instance = binding.alimp_instance
                # The height is inflated: 1 for input buffer, 1 for output buffer, 1 for the transporters attached to output buffer, and 1 on each side for routing space.
                # The width is inflated: 1 on each side for routing space.
                shape = ds.RectangleShape(width=instance.width+2*db.hyper_parameter.place_reserved_routing_size, height=instance.height+3+2*db.hyper_parameter.place_reserved_routing_size)
                fp.shape.append(shape)
                fp.pos.append(ds.RectanglePosion(x=-1, y=-1))
                break
        node_map[node.id] = len(fp.app_node_ids)
        fp.app_node_ids.append(node.id)

    # for each edge in app_graph, find the corresponding edge connection weight and modify fp.conn
    for edge in db.app_graph.edges:
        fp.app_edge_ids.append(edge.id)
        fp.source_node.append(node_map[edge.source_node])
        fp.target_node.append(node_map[edge.target_node])

        # find the index of the port in the output port list of the source node
        source_port_index = -1
        for i in range(len(db.app_graph.nodes)):
            if db.app_graph.nodes[i].id == edge.source_node:
                for j in range(len(db.app_graph.nodes[i].output_ports)):
                    if db.app_graph.nodes[i].output_ports[j].id == edge.source_port:
                        source_port_index = j
                        break
                break
        if source_port_index == -1:
            logging.error('Cannot find the source port %s in node %s' % (edge.source_port, edge.source_node))
            exit(1)
        
        fp.source_port.append(source_port_index)

        # find the index of the port in the input port list of the target node
        target_port_index = -1
        for i in range(len(db.app_graph.nodes)):
            if db.app_graph.nodes[i].id == edge.target_node:
                for j in range(len(db.app_graph.nodes[i].input_ports)):
                    if db.app_graph.nodes[i].input_ports[j].id == edge.target_port:
                        target_port_index = j
                        break
                break
        if target_port_index == -1:
            logging.error('Cannot find the target port %s in node %s' % (edge.target_port, edge.target_node))
            exit(1)

        fp.target_port.append(target_port_index)

        fp.conn.append(edge.token_size)

    return fp

def place_solve_optimal(fp: ds.FloorPlan) -> bool:
    ''' Solve the placement problem optimally. It's a 2D bin-packing problem. This function returns true if it's successful. '''
    
    num_squares = len(fp.shape)

    model = cp_model.CpModel()
    x_intervals = []
    y_intervals = []
    x_starts = []
    y_starts = []
    x_end = []
    y_end = []

    # Creates intervals for the NoOverlap2D and size variables.
    for i in range(num_squares):
        start_x = model.NewIntVar(0, fp.max_width, "sx_%i" % i)
        end_x = model.NewIntVar(0, fp.max_width, "ex_%i" % i)
        start_y = model.NewIntVar(0, fp.max_height, "sy_%i" % i)
        end_y = model.NewIntVar(0, fp.max_height, "ey_%i" % i)

        interval_x = model.NewIntervalVar(
                start_x, fp.shape[i].width, end_x, "ix_%i" % i)
        interval_y = model.NewIntervalVar(
                start_y, fp.shape[i].height, end_y, "iy_%i" % i)

        x_intervals.append(interval_x)
        y_intervals.append(interval_y)
        x_starts.append(start_x)
        y_starts.append(start_y)
        x_end.append(end_x)
        y_end.append(end_y)

    # Main constraint.
    model.AddNoOverlap2D(x_intervals, y_intervals)

    # Symmetry breaking 2: first square in one quadrant.
    model.Add(x_starts[0] < (fp.max_width + 1) // 2)
    model.Add(y_starts[0] < (fp.max_height + 1) // 2)

    # Compute the maximum x and y positions.
    max_x_position = model.NewIntVar(0, fp.max_width, "max_x_position")
    model.AddMaxEquality(max_x_position, [x for x in x_end])
    max_y_position = model.NewIntVar(0, fp.max_height, "max_y_position")
    model.AddMaxEquality(max_y_position, [y for y in y_end])

    # add a restriction of the ration between max_x_position and max_y_position. the ratio should be greater than 0.5 and less than 2
    model.Add(max_x_position * 2 >= max_y_position)
    model.Add(max_x_position <= max_y_position * 2)

    # Compute the total area=max_x_position*max_y_position.
    total_area = model.NewIntVar(
            0, fp.max_width * fp.max_height, "total_area")
    model.AddMultiplicationEquality(
            total_area, [max_x_position, max_y_position])

    model.Minimize(total_area)

    # Creates a solver and solves.
    solver = cp_model.CpSolver()
    solver.parameters.num_workers = 16
    # solver.parameters.log_search_progress = True
    solver.parameters.max_time_in_seconds = 10.0
    status = solver.Solve(model)
    solution_found = status == cp_model.OPTIMAL

    if solution_found:
            fp.pos.clear()
            for i in range(num_squares):
                p = ds.RectanglePosion()
                p.x = solver.Value(x_starts[i])
                p.y = solver.Value(y_starts[i])
                fp.pos.append(p)
                print(p.x, p.y, fp.shape[i].width, fp.shape[i].height)
            max_width = solver.Value(max_x_position)
            max_height = solver.Value(max_y_position)

            fp.max_width = max_width
            fp.max_height = max_height

    return solution_found

def place_solve_approx_optimal(fp: ds.FloorPlan, db:ds.DataBase) -> bool:
    x_sizes = [r.width for r in fp.shape]
    y_sizes = [r.height for r in fp.shape]
    num_squares = len(x_sizes)

    """Try to fill the rectangle with a given number of squares."""
    size_x = fp.max_width
    size_y = fp.max_height

    model = cp_model.CpModel()

    areas = []
    x_intervals = []
    y_intervals = []
    x_starts = []
    y_starts = []

    # Creates intervals for the NoOverlap2D and size variables.
    for i in range(num_squares):
            start_x = model.NewIntVar(0, size_x, "sx_%i" % i)
            end_x = model.NewIntVar(0, size_x, "ex_%i" % i)
            start_y = model.NewIntVar(0, size_y, "sy_%i" % i)
            end_y = model.NewIntVar(0, size_y, "ey_%i" % i)

            interval_x = model.NewIntervalVar(
                start_x, x_sizes[i], end_x, "ix_%i" % i)
            interval_y = model.NewIntervalVar(
                start_y, y_sizes[i], end_y, "iy_%i" % i)

            area = x_sizes[i] * y_sizes[i]
            areas.append(area)
            x_intervals.append(interval_x)
            y_intervals.append(interval_y)
            x_starts.append(start_x)
            y_starts.append(start_y)

    # Main constraint.
    model.AddNoOverlap2D(x_intervals, y_intervals)

    # Symmetry breaking 2: first square in one quadrant.
    model.Add(x_starts[0] < (size_x + 1) // 2)
    model.Add(y_starts[0] < (size_y + 1) // 2)

    # Symmetry breaking 3: minimal value in start_x equals to zero and minimal value in start_y equals to zero.
    model.AddMinEquality(0, x_starts)
    model.AddMinEquality(0, y_starts)

    # Compute the distance matrix between each pair of squares. The distance is Manhattan distance from the north of the source to the south of the target.
    distance_matrix = []
    for i in range(len(fp.source_node)):
        source_node = fp.source_node[i]
        target_node = fp.target_node[i]
        source_port = fp.source_port[i]
        target_port = fp.target_port[i]
        source_x = x_starts[source_node] + source_port + db.hyper_parameter.place_reserved_routing_size
        source_y = y_starts[source_node] + y_sizes[source_node] + 2* db.hyper_parameter.place_reserved_routing_size
        target_x = x_starts[target_node] + target_port + db.hyper_parameter.place_reserved_routing_size
        target_y = y_starts[target_node] + db.hyper_parameter.place_reserved_routing_size
        distance_matrix.append(model.NewIntVar(
            0, size_x + size_y, "dist_%i_%i" % (source_node, target_node)))
        dx_0 = model.NewIntVar(-size_x, size_x, "dx0_%i_%i" % (source_node, target_node))
        model.Add(dx_0 == source_x - target_x)
        dx = model.NewIntVar(0, size_x, "dx_%i_%i" % (source_node, target_node))
        model.AddAbsEquality(dx, dx_0)
        dy_0 = model.NewIntVar(-size_y, size_y, "dy0_%i_%i" % (source_node, target_node))
        model.Add(dy_0 == source_y - target_y)
        dy = model.NewIntVar(0, size_y, "dy_%i_%i" % (source_node, target_node))
        model.AddAbsEquality(dy, dy_0)
        model.Add(distance_matrix[i] == dx + dy)

    # Compute the weighted distance by multiplying the distance matrix by the connectivity matrix.
    weighted_distance = []
    for i in range(len(fp.source_node)):
                weighted_distance.append(model.NewIntVar(
                    0, 10000, "wdist_%i_%i" % (i, i)))
                model.AddMultiplicationEquality(weighted_distance[i],
                                                [distance_matrix[i], fp.conn[i]])

    # Compute the total weighted distance.
    total_weighted_distance = model.NewIntVar(
            0, 10000, "total_weighted_distance")
    model.Add(sum(weighted_distance) == total_weighted_distance)

    # Objective: minimize the total weighted distance.
    model.Minimize(total_weighted_distance)

    # Compute the maximum x and y positions.
    max_x_position = model.NewIntVar(0, size_x, "max_x_position")
    model.AddMaxEquality(
            max_x_position, [interval.EndExpr() for interval in x_intervals])
    max_y_position = model.NewIntVar(0, size_y, "max_y_position")
    model.AddMaxEquality(
            max_y_position, [interval.EndExpr() for interval in y_intervals])
    
    # add a restriction of the ration between max_x_position and max_y_position. the ratio should be greater than 0.5 and less than 2
    model.Add(max_x_position * 2 >= max_y_position)
    model.Add(max_x_position <= max_y_position * 2)

    # Creates a solver and solves.
    solver = cp_model.CpSolver()
    solver.parameters.num_workers = 16
    # solver.parameters.log_search_progress = True
    solver.parameters.max_time_in_seconds = 10.0
    status = solver.Solve(model)
    solution_found = status == cp_model.OPTIMAL or status == cp_model.FEASIBLE
    if solution_found:
            fp.pos.clear()
            for i in range(num_squares):
                p = ds.RectanglePosion()
                p.x = solver.Value(x_starts[i])
                p.y = solver.Value(y_starts[i])
                fp.pos.append(p)

            max_width = solver.Value(max_x_position)
            max_height = solver.Value(max_y_position)

            fp.max_width = max_width
            fp.max_height = max_height
    return solution_found

# This function generate a picture of the floorplan. It accepts four parameters:
# - max_x: the maximum x position of the floorplan
# - max_y: the maximum y position of the floorplan
# - start: the start position of each rectangle. Each rectangle is a list of two elements: the x position and the y position
# - size: the size of each rectangles. Each rectangle is a list of two elements: the width and the height of the rectangle
# In the generated picture, each rectangle is represented by a different light color. The color is generated randomly. The grid is also shown in the picture as dashed line.
# It also mark the index of each rectangle in bold font in the center of each drawed rectangle. The color of the font is the opposite of the color of the rectangle.

def generate_picture(fp: ds.FloorPlan, db:ds.DataBase, output_dir:str) -> None:
        import matplotlib.pyplot as plt
        import matplotlib.patches as patches
        import random
        max_x = fp.max_width
        max_y = fp.max_height
        start = fp.pos
        size = fp.shape
        fig = plt.figure()
        ax = fig.add_subplot(111, aspect='equal')
        ax.set_xlim([0, max_x])
        ax.set_ylim([0, max_y])
        for i in range(len(start)):
            x = random.random()
            y = random.random()
            z = random.random()
            ax.add_patch(patches.Rectangle(
                (start[i].x+db.hyper_parameter.place_reserved_routing_size , start[i].y+1+db.hyper_parameter.place_reserved_routing_size), size[i].width-2*db.hyper_parameter.place_reserved_routing_size, size[i].height-3-2*db.hyper_parameter.place_reserved_routing_size, color=(x, y, z)))
            ax.text(start[i].x + size[i].width / 2, start[i].y + size[i].height / 2, fp.app_node_ids[i], horizontalalignment='center', verticalalignment='center', weight='bold', color=(1 - x, 1 - y, 1 - z))
        plt.grid(True, linestyle='--')
        # save the picture to pdf file
        plt.savefig(os.path.join(output_dir, "placement.pdf"), bbox_inches='tight')

def run(db: ds.DataBase, output_dir) -> bool:
    ''' Run the placement process. '''
    logging.info("Start: placement")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    module_dir = os.path.join(output_dir, 'place')
    os.makedirs(module_dir, exist_ok=True)

    # create floor plan
    fp = create_floor_plan(db)

    # solve the placement problem
    if not place_solve_optimal(fp):
        logging.warning('Cannot find a optimal solution for the placement problem')
        return False
    
    # relax width and height constraint
    print(fp.max_width, fp.max_height)
    fp.max_width = int(fp.max_width * db.hyper_parameter.place_relaxation_factor)
    fp.max_height = int(fp.max_height * db.hyper_parameter.place_relaxation_factor)

    # solve the placement problem again
    if not place_solve_approx_optimal(fp, db):
        logging.warning('Cannot find an approx optimal solution for the placement problem')
        return False

    # generate picture
    generate_picture(fp, db, module_dir)

    # update placement
    for i in range(len(fp.app_node_ids)):
        placement = ds.Placement()
        placement.app_node_id = fp.app_node_ids[i]
        placement.x = fp.pos[i].x + 1
        placement.y = fp.pos[i].y + 2
        db.synthesized_information.placements.append(placement)
    
    # update max_width and max_height
    db.synthesized_information.max_width = fp.max_width
    db.synthesized_information.max_height = fp.max_height

    logging.info("Finish: placement")
    return True