#!/usr/bin/env python3
# -*- coding: utf-8 -*-

## @package bind
# This module contains process to bind app_graph actors to implementations in alimp_lib
# It uses or-tools to check the feasibility of the binding against the global constraint
# It does not choose the best binding. It will instead keep multiple promissing bindings and sort them based on a weighted cost function.

import os
import sys
import logging

import lib.proto.data_structure_pb2 as ds
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

from ortools.constraint_solver import pywrapcp
from ortools.sat.python import cp_model


class Callback(cp_model.CpSolverSolutionCallback):

    def __init__(self, variables):
        cp_model.CpSolverSolutionCallback.__init__(self)
        self.variables = variables
        self.solutions = []

    def on_solution_callback(self):
        solution = {}
        for k,v in self.variables.items():
            solution[k] = self.Value(v)
        self.solutions.append(solution)

def bind_solve_optimal (db: ds.DataBase) -> int:
    model = cp_model.CpModel()
    
    # Create binding variable for each node in the app_graph, the binding variable is the index of the implementation in the alimp_lib.
    # Create a one-hot encoded binding vector for each binding variable.

    # Geometry constraint:
        # The width and height of any alimp cannot be larger than the global width and height
        # The area of all selected alimps cannot be larger than the global area
    
    # Energy constraint:
        # The energy of all selected alimps cannot be larger than the global energy

    BINDING_VARS = {}
    BINDING_VECS = {}
    ALIMP_AREA = {}
    ALIMP_ENERGY = {}
    for node in db.app_graph.nodes:
        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        BINDING_VARS[node.id] = model.NewIntVar(0, len(alimp_entry.instances) - 1, node.id)
        BINDING_VECS[node.id] = []
        ALIMP_AREA[node.id] = model.NewIntVar(0, db.global_constraint.max_width * db.global_constraint.max_height, node.id + "_area")
        ALIMP_ENERGY[node.id] = model.NewIntVar(0, db.global_constraint.max_energy, node.id + "_energy")
        for i in range(len(alimp_entry.instances)):
            BINDING_VECS[node.id].append(model.NewBoolVar(node.id + "_" + str(i)))
            model.Add(BINDING_VARS[node.id] == i).OnlyEnforceIf(BINDING_VECS[node.id][i])

            # Geometry constraint
            model.Add(alimp_entry.instances[i].width <= db.global_constraint.max_width).OnlyEnforceIf(BINDING_VECS[node.id][i])
            model.Add(alimp_entry.instances[i].height <= db.global_constraint.max_height).OnlyEnforceIf(BINDING_VECS[node.id][i])
            model.Add(alimp_entry.instances[i].width * alimp_entry.instances[i].height == ALIMP_AREA[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])

            # Energy constraint
            model.Add(alimp_entry.instances[i].energy == ALIMP_ENERGY[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])

        
        model.Add(sum(BINDING_VECS[node.id]) == 1)
        # Geometry constraint
        model.Add(sum(ALIMP_AREA.values()) <= db.global_constraint.max_width * db.global_constraint.max_height)

        # Energy constraint
        model.Add(sum(ALIMP_ENERGY.values()) <= db.global_constraint.max_energy)  

    # Now we consider latency constraint
    # First, create a predecessor list for each node
    predecessors = {}
    for edge in db.app_graph.edges:
        node = None
        for n in db.app_graph.nodes:
            if n.id == edge.source_node:
                node = n
                break
        if node is None:
            logging.error("Cannot find node %s", edge.source_node)
            sys.exit(1)
        if node.id not in predecessors:
            predecessors[node.id] = []
        predecessors[node.id].append(edge.target_node)

    # create a start time variable for each node
    START_TIME = {}
    LATENCY = {}
    END_TIME = {}
    HALF_LATENCY = {}

    for node in db.app_graph.nodes:
        START_TIME[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_start_time")
        LATENCY[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_latency")
        HALF_LATENCY[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_half_latency")
        # post constraint: start_time is the max of all predecessors' start_time + 0.5*latency
        model.AddDivisionEquality(HALF_LATENCY[node.id], LATENCY[node.id], 2)
    for node in db.app_graph.nodes:
        if node.id in predecessors: 
            max_predecessor_time = model.NewIntVar(0, db.global_constraint.max_latency, f'max_predecessor_time_{node.id}')
            predecessor_end_times = []
            for predecessor in predecessors[node.id]:
                pred_end = START_TIME[predecessor] + HALF_LATENCY[predecessor]
                predecessor_end_times.append(pred_end)
            # post constraint: start_time is the max of all predecessors' start_time + 0.5*latency
            model.AddMaxEquality(max_predecessor_time, predecessor_end_times)
            model.Add(START_TIME[node.id] == max_predecessor_time)
        else:
            model.Add(START_TIME[node.id] == 0)
        # post constraint: start_time + latency <= max_latency
        model.Add(START_TIME[node.id] + LATENCY[node.id] <= db.global_constraint.max_latency)
        # post constraint: latency is the alimp instance latency when the alimp is selected

        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        for i in range(len(alimp_entry.instances)):
            model.Add(alimp_entry.instances[i].latency == LATENCY[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])
        
        # post constraint: end_time = start_time + latency
        END_TIME[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_end_time")
        model.Add(END_TIME[node.id] == START_TIME[node.id] + LATENCY[node.id])
    
    # Throughput constraint: Any alimp instance should have a latency that is smaller than the max period
    for node in db.app_graph.nodes:
        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        for i in range(len(alimp_entry.instances)):
            model.Add(alimp_entry.instances[i].latency <= db.global_constraint.max_period).OnlyEnforceIf(BINDING_VECS[node.id][i])
    
    # define the objective function, which is the weighted linear combination of area, energy and latency
    obj = model.NewIntVar(0, 1000000, "obj")
    model.Add(obj == db.hyper_parameter.bind_w_area * sum(ALIMP_AREA.values()) + db.hyper_parameter.bind_w_energy *sum(ALIMP_ENERGY.values()))

    # find the optimal solution
    model.Minimize(obj)


    # Solve the model.
    solver = cp_model.CpSolver()
    status = solver.Solve(model)
    if status == cp_model.OPTIMAL:
        # get the optimal obj value
        optimal_obj = solver.ObjectiveValue()
        return optimal_obj
    else:
        return None


def bind_solve_approx_optimal(db: ds.DataBase, obj_optimal) -> list:
    ''' This function create the same model as bind_solve_optimal, but it relax the obj function and uses the cp_model.SearchForAllSolutions() to find all feasible solutions. It returns a list of feasible solutions.'''
    print("obj_optimal =", obj_optimal)
    relaxted_obj = int(obj_optimal * db.hyper_parameter.bind_relaxation_factor)
    if relaxted_obj == int(obj_optimal):
        relaxted_obj = 2 * int(obj_optimal)
    
    # Create the model
    model = cp_model.CpModel()

# Create binding variable for each node in the app_graph, the binding variable is the index of the implementation in the alimp_lib.
    # Create a one-hot encoded binding vector for each binding variable.

    # Geometry constraint:
        # The width and height of any alimp cannot be larger than the global width and height
        # The area of all selected alimps cannot be larger than the global area
    
    # Energy constraint:
        # The energy of all selected alimps cannot be larger than the global energy

    BINDING_VARS = {}
    BINDING_VECS = {}
    ALIMP_AREA = {}
    ALIMP_ENERGY = {}
    for node in db.app_graph.nodes:
        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        BINDING_VARS[node.id] = model.NewIntVar(0, len(alimp_entry.instances) - 1, node.id)
        BINDING_VECS[node.id] = []
        ALIMP_AREA[node.id] = model.NewIntVar(0, db.global_constraint.max_width * db.global_constraint.max_height, node.id + "_area")
        ALIMP_ENERGY[node.id] = model.NewIntVar(0, db.global_constraint.max_energy, node.id + "_energy")
        for i in range(len(alimp_entry.instances)):
            BINDING_VECS[node.id].append(model.NewBoolVar(node.id + "_" + str(i)))
            model.Add(BINDING_VARS[node.id] == i).OnlyEnforceIf(BINDING_VECS[node.id][i])

            # Geometry constraint
            model.Add(alimp_entry.instances[i].width <= db.global_constraint.max_width).OnlyEnforceIf(BINDING_VECS[node.id][i])
            model.Add(alimp_entry.instances[i].height <= db.global_constraint.max_height).OnlyEnforceIf(BINDING_VECS[node.id][i])
            model.Add(alimp_entry.instances[i].width * alimp_entry.instances[i].height == ALIMP_AREA[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])

            # Energy constraint
            model.Add(alimp_entry.instances[i].energy == ALIMP_ENERGY[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])

        
        model.Add(sum(BINDING_VECS[node.id]) == 1)
        # Geometry constraint
        model.Add(sum(ALIMP_AREA.values()) <= db.global_constraint.max_width * db.global_constraint.max_height)

        # Energy constraint
        model.Add(sum(ALIMP_ENERGY.values()) <= db.global_constraint.max_energy)  

    # Now we consider latency constraint
    # First, create a predecessor list for each node
    predecessors = {}
    for edge in db.app_graph.edges:
        node = None
        for n in db.app_graph.nodes:
            if n.id == edge.source_node:
                node = n
                break
        if node is None:
            logging.error("Cannot find node %s", edge.source_node)
            sys.exit(1)
        if node.id not in predecessors:
            predecessors[node.id] = []
        predecessors[node.id].append(edge.target_node)

    # create a start time variable for each node
    START_TIME = {}
    LATENCY = {}
    END_TIME = {}
    HALF_LATENCY = {}

    for node in db.app_graph.nodes:
        START_TIME[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_start_time")
        LATENCY[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_latency")
        HALF_LATENCY[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_half_latency")
        # post constraint: start_time is the max of all predecessors' start_time + 0.5*latency
        model.AddDivisionEquality(HALF_LATENCY[node.id], LATENCY[node.id], 2)
    for node in db.app_graph.nodes:
        if node.id in predecessors:
            max_predecessor_time = model.NewIntVar(0, db.global_constraint.max_latency, f'max_predecessor_time_{node.id}')
            predecessor_end_times = []
            for predecessor in predecessors[node.id]:
                pred_end = START_TIME[predecessor] + HALF_LATENCY[predecessor]
                predecessor_end_times.append(pred_end)
            # post constraint: start_time is the max of all predecessors' start_time + 0.5*latency
            model.AddMaxEquality(max_predecessor_time, predecessor_end_times)
            model.Add(START_TIME[node.id] == max_predecessor_time)
        else:
            model.Add(START_TIME[node.id] == 0)
        # post constraint: start_time + latency <= max_latency
        model.Add(START_TIME[node.id] + LATENCY[node.id] <= db.global_constraint.max_latency)
        # post constraint: latency is the alimp instance latency when the alimp is selected

        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        for i in range(len(alimp_entry.instances)):
            model.Add(alimp_entry.instances[i].latency == LATENCY[node.id]).OnlyEnforceIf(BINDING_VECS[node.id][i])
        
        # post constraint: end_time = start_time + latency
        END_TIME[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, node.id + "_end_time")
        model.Add(END_TIME[node.id] == START_TIME[node.id] + LATENCY[node.id])
    
    # Throughput constraint: Any alimp instance should have a latency that is smaller than the max period
    for node in db.app_graph.nodes:
        alimp_entry = None
        for entry in db.alimp_lib.entries:
            if entry.func == node.func:
                alimp_entry = entry
                break
        if alimp_entry is None:
            logging.error("Cannot find alimp entry for function %s", node.func)
            sys.exit(1)
        for i in range(len(alimp_entry.instances)):
            model.Add(alimp_entry.instances[i].latency <= db.global_constraint.max_period).OnlyEnforceIf(BINDING_VECS[node.id][i])
    
    # define the objective function, which is the weighted linear combination of area, energy and latency
    obj = model.NewIntVar(0, 2 * 1000000, "obj")
    model.Add(obj == db.hyper_parameter.bind_w_area * sum(ALIMP_AREA.values()) + db.hyper_parameter.bind_w_energy *sum(ALIMP_ENERGY.values()))

    # add constraint to limit the max value of obj to relatexed_obj
    model.Add(obj <= relaxted_obj)

    # Find all feasible solutions. For each solution, store the binding variables (BINDING_VARS) in a list
    solver = cp_model.CpSolver()
    callback = Callback(BINDING_VARS)
    solver.SearchForAllSolutions(model, callback)

    return callback.solutions

def apply_binding_options(db: ds.DataBase, binding_options: dict):
    ''' This function applies the binding to the app_graph and alimp_lib '''
    del db.alimp_binding_options[:]

    # for each binding option:
    for option in binding_options:
        db.alimp_binding_options.append(ds.AlimpBindingOption())
        # apply binding to app_graph
        for node in db.app_graph.nodes:
            alimp_entry = None
            for entry in db.alimp_lib.entries:
                if entry.func == node.func:
                    alimp_entry = entry
                    break
            if alimp_entry is None:
                logging.error("Cannot find alimp entry for function %s", node.func)
                sys.exit(1)
            
            app_node_id = node.id
            alimp_instance = alimp_entry.instances[option[node.id]]
            db.alimp_binding_options[-1].alimp_bindings.append(ds.AlimpBinding(app_node_id=app_node_id, alimp_instance=alimp_instance))

def create_app_graph(db: ds.DataBase, output_dir: str):
    ''' This function creates a graphviz representation of the app_graph and saves it to a file '''
    import graphviz
    dot = graphviz.Digraph(comment='App Graph')
    for node in db.app_graph.nodes:
        dot.node(node.id, node.id)
    for edge in db.app_graph.edges:
        dot.edge(edge.source_node, edge.target_node)
    dot.render(os.path.join(output_dir, 'app_graph.gv'), view=False)

def run(db: ds.DataBase, output_dir: str):
    logging.info("Start: binding")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    module_dir = os.path.join(output_dir, 'bind')
    os.makedirs(module_dir, exist_ok=True)

    # create application graph
    create_app_graph(db, module_dir)

    obj = bind_solve_optimal(db)
    if obj is None:
        logging.error("Cannot find optimal solution, try to relax the global constraints")
        sys.exit(1)
    logging.info("Optimal obj value: %d", obj)
    logging.info("Obj will be relaxed by relaxation factor: %f", db.hyper_parameter.bind_relaxation_factor)
    valid_bindings = bind_solve_approx_optimal(db, obj)
    if len(valid_bindings) == 0:
        logging.error("Cannot find any valid binding")
        sys.exit(1)
    logging.info("Found solutions:")
    print(valid_bindings)
    apply_binding_options(db, valid_bindings)
    for x in db.alimp_binding_options[0].alimp_bindings:
        db.synthesized_information.alimp_bindings.append(x)
    logging.info("Finish: binding")
