import os
import sys
import logging
import numpy as np
import math
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import lib.proto.data_structure_pb2 as ds

from ortools.sat.python import cp_model

def merge_two_dicts(x, y):
    z = x.copy()   # start with keys and values of x
    z.update(y)    # modifies z with keys and values of y
    return z

def pair_int_int_to_dict(pair_list: list) -> dict:
    d = {}
    for pair in pair_list:
        d[pair.key] = pair.value
    return d
'''
def create_app_graph() -> ds.AppGraph:
    g = ds.AppGraph()
    node = ds.AppNode(id="conv_3x3_1", func="conv_3x3",
                      repetition=1, execution_time=4)
    node.output_ports.append(ds.AppNodePort(id="conv_3x3_1_output", rate=1, token_size=5, addr_time_patterns=[{"key": 0, "value": 1}, {
                             "key": 1, "value": 2}, {"key": 2, "value": 3}, {"key": 3, "value": 4}, {"key": 4, "value": 5}]))
    g.nodes.append(node)
    node = ds.AppNode(id="conv_3x3_2", func="conv_3x3",
                      repetition=1, execution_time=6)
    node.input_ports.append(ds.AppNodePort(id="conv_3x3_2_input", rate=1, token_size=5, addr_time_patterns=[{"key": 0, "value": 0}, {
                            "key": 1, "value": 1}, {"key": 2, "value": 2}, {"key": 3, "value": 3}, {"key": 4, "value": 4}]))
    g.nodes.append(node)
    edge = ds.AppEdge(source_node="conv_3x3_1", target_node="conv_3x3_2", source_port="conv_3x3_1_output",
                      target_port="conv_3x3_2_input", token_size=5, delay=4)
    g.edges.append(edge)
    return g
'''

def stats_addr_patterns(pattern1: dict, pattern2: dict) -> [int, int, int, int]:
    # find the common address of both src_addr_pattern and dest_addr_pattern and store in a set
    common_addr = set()
    for addr in pattern1:
        if addr in pattern2:
            common_addr.add(addr)

    diff_pattern = []
    cycles1 = []
    cycles2 = []
    for addr in common_addr:
        diff_pattern.append(abs(pattern2[addr] - pattern1[addr]))
        cycles1.append(pattern1[addr])
        cycles2.append(pattern2[addr])

    cycle_counts1 = {}
    cycle_counts2 = {}
    for cycle in cycles1:
        if cycle not in cycle_counts1:
            cycle_counts1[cycle] = 0
        cycle_counts1[cycle] += 1
    for cycle in cycles2:    
        if cycle not in cycle_counts2:
            cycle_counts2[cycle] = 0
        cycle_counts2[cycle] += 1
    addresses_per_cycle1 = list(cycle_counts1.values())
    addresses_per_cycle2 = list(cycle_counts2.values())
    mean1, mean2 = np.mean(addresses_per_cycle1), np.mean(addresses_per_cycle2)
    mean = (mean1 + mean2) / 2
    std1, std2 = np.std(addresses_per_cycle1), np.std(addresses_per_cycle2)
    std = (std1 + std2) / 2

    return min(diff_pattern), max(diff_pattern), mean, std


def find_min_delay(src_addr_pattern: dict, dest_addr_pattern: dict, channel_width: int, max_channel_width: int, max_delay: int) -> int:
    # find the common address of both src_addr_pattern and dest_addr_pattern and store in a set
    common_addr = set()
    for addr in src_addr_pattern:
        if addr in dest_addr_pattern:
            common_addr.add(addr)
    
    #print(f"create graph (max delay = {max_delay}, channel_width = {channel_width})")
    # Use OR-tools to find the minimum delay
    model = cp_model.CpModel()
    # create variables
    delay = model.NewIntVar(2, max_delay, "delay")
   # This constraint needs to take a look more 
   ## add a quick heuristic estimation 
   # min_cycles, max_cycles, mean_address, std_address = stats_addr_patterns(
   #         src_addr_pattern, 
   #         dest_addr_pattern
   # )
   # model.add(delay >= max_cycles + 2)
   # if (len(common_addr) > 400):
   #     estimated_cycles = len(common_addr) / channel_width
   #     pipeline_cycles = len(common_addr) / mean_address
   #     estimated_delay = max_cycles + 2 + round(estimated_cycles - pipeline_cycles)
   #     #print(f"estimated_delay = {estimated_delay}, min_value = {min_cycles}, max_value = {max_cycles}, mean_address = {mean_address}, std_address = {std_address}")
   #     model.add(delay <= estimated_delay + 300)
   #     model.add(delay >= estimated_delay - 20)

    # create TRANSPORTER VARIABLES for READ for each address
    tr_read = {}
    for addr in common_addr:
        earliest_start = src_addr_pattern[addr] + 1
        tr_read[addr] = model.NewIntVar(earliest_start, max_delay, f"tr_read_{addr}")
    # add constraints
    for addr in common_addr:
        model.Add(tr_read[addr] > src_addr_pattern[addr])
        model.Add(dest_addr_pattern[addr] + delay > tr_read[addr])
    
    # cummulative constraint
    interval = [model.NewIntervalVar(tr_read[addr], 1, tr_read[addr] + 1, f"interval[{addr}]") for addr in common_addr]
    model.AddCumulative(interval, [1 for addr in common_addr], channel_width)
    
    # objective
    model.Minimize(delay)
    # solve
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 30.0 
    #solver.parameters.log_search_progress = True
    solver.parameters.linearization_level = 0
    solver.parameters.max_number_of_conflicts = 5000    
    solver.parameters.max_memory_in_mb = 8000  
    solver.parameters.randomize_search = True
    solver.parameters.random_seed = 100
    solver.parameters.use_lns = True

    status = solver.Solve(model)
    if status == cp_model.OPTIMAL or status == cp_model.FEASIBLE:   
        #print("DELAY=", solver.Value(delay))
        #print("SRC=", [src_addr_pattern[addr] for addr in common_addr])
        #print("DEST=", [dest_addr_pattern[addr] for addr in common_addr])
        #print("TR_READ=", [solver.Value(tr_read[addr]) for addr in common_addr])
        return solver.Value(delay)
    else:
        #print("Cannot find a delay value")
        #logging.error("Cannot find a delay value for the provided address time patterns")
        return -1   


def find_parallelization_degree(addr_pattern: dict) -> int:
    # return the max count of the same value in addr_pattern
    max_count = 0
    for addr in addr_pattern:
        count = sum([1 for x in addr_pattern if addr_pattern[addr] == addr_pattern[x]])
        if count > max_count:
            max_count = count
    return max_count

def translate_addr_time_patterns(db, target_node, port_name, pattern_dir):
    for node in db.synthesized_information.alimp_bindings:
        if node.app_node_id == target_node:
            if pattern_dir == 'in':
                patterns = node.alimp_instance.input_addr_time_patterns
            else:
                patterns = node.alimp_instance.output_addr_time_patterns
            patterns = pair_int_int_to_dict(patterns)
            break
    else:
        logging.error("Cannot find the address time patterns in Alimp")
        sys.exit(1)
    
    # select only patterns for the target node
    for node in db.app_graph.nodes:
        if node.id == target_node:
            if pattern_dir == 'in':
                ports = node.input_ports
            else:
                ports = node.output_ports
            start_address = 0
            for port in ports:
                if port.id == port_name:
                    range_address = port.token_size 
                    addr_time_patterns = {}
                    for addr in patterns.keys():
                        if (addr - start_address >= 0) and (addr - start_address < range_address):
                            addr_time_patterns[addr - start_address] = patterns[addr]
                    break
                else:
                    start_address += port.token_size
            else:
                logging.error("Cannot find the port in ports")
                sys.exit(1)
            break
    else:
        logging.error("Cannot find the node in graphs")
        sys.exit(1)
    return addr_time_patterns



def translate_source_addr(app_graph, source_node, source_port, addr):
    source_addr = 0
    for node in app_graph.nodes:
        if node.id == source_node:
            for port in node.output_ports:
                if port.id == source_port:
                    source_addr += addr
                    break
                else:
                    source_addr += port.token_size
            break
    return source_addr


def translate_target_addr(app_graph, target_node, target_port, addr):
    target_addr = 0
    for node in app_graph.nodes:
        if node.id == target_node:
            for port in node.input_ports:
                if port.id == target_port:
                    target_addr += addr
                    break
                else:
                    target_addr += port.token_size
            break
    return target_addr


def optimize_channel_width(db: ds.DataBase):
    model = cp_model.CpModel()
    # create int variable for the fire time of each node
    F = {}
    all_nodes = []
    END_TIME = {}
    for node in db.app_graph.nodes:
        all_nodes.append(node.id)
        F[node.id] = [model.NewIntVar(0, db.global_constraint.max_latency, f"{node.id}_{i}")
                      for i in range(node.repetition)]
        END_TIME[node.id] = model.NewIntVar(0, db.global_constraint.max_latency, f"{END_TIME}_{node.id}")
        model.Add(END_TIME[node.id] == F[node.id][node.repetition-1]+node.execution_time)
    # the minimal of F should be 0: Do we need this? The variables themselves are declared from 0 ... somewhere
    model.AddMinEquality(0, [F[node.id][0] for node in db.app_graph.nodes]) 
    # This variable should not be modeled
    ## for each edge, create a variable for the fire time of the transporter
    #for edge in db.app_graph.edges:
    #    F["transporter_"+edge.source_port + "_" +
    #        edge.target_port] = [model.NewIntVar(0, db.global_constraint.max_latency, "transporter_"+edge.source_port + "_" +
    #                                             edge.target_port+"_0")]
    
    K_VARS = {}
    K_VECS = {}
    DISTANCE = {}
    MIN_DELAY = {}
    # for each edge, find the min delay for each channel width
    # At this stage, we don't have to consider wire delay
    for edge in db.app_graph.edges: 
        output_addr_time_patterns = translate_addr_time_patterns(db, edge.source_node, 
                                                                 edge.source_port, 'out')

        input_addr_time_patterns = translate_addr_time_patterns(db, edge.target_node, 
                                                                edge.target_port, 'in')
    
        for route in db.synthesized_information.routing_paths:
            if route.app_edge_id == edge.id:
                DISTANCE[edge.id] = len(route.path)
                break
        else:
            logging.error("Cannot find an edge in the routing paths")
            sys.exit(1)
               
        max_channel_width = max(find_parallelization_degree(output_addr_time_patterns), 
                                find_parallelization_degree(input_addr_time_patterns))
        K_VARS[edge.id] = model.NewIntVar(1, max_channel_width, "K_VARS_"+edge.id)
        K_VECS[edge.id] = []
        MIN_DELAY[edge.id] = []
        min_cycles, max_cycles, mean_address, std_address = stats_addr_patterns(
            output_addr_time_patterns, 
            input_addr_time_patterns
        )
        print(f"edge = {edge.id}, max_channel_width = {max_channel_width}, mean = {mean_address}")
        no_improvement = False
        for k in range (max_channel_width):
            K_VECS[edge.id].append(model.NewBoolVar("K_VECS_"+edge.id+"_"+str(k)))
            model.Add(K_VARS[edge.id] == k + 1).OnlyEnforceIf(K_VECS[edge.id][k])
            # to find min delay if k >= mean - 1
            if (k + 1) < math.floor(mean_address) - 1:
                model.Add(K_VECS[edge.id][k] == 0)
                MIN_DELAY[edge.id].append(-1)
                continue
            # detect no improvement for 3 times
            if len(MIN_DELAY[edge.id]) > 2:
                if (len(set(MIN_DELAY[edge.id][-3:])) == 1 and MIN_DELAY[edge.id][-1] != -1) or no_improvement:
                    no_improvement = True
                    model.Add(K_VECS[edge.id][k] == 0)
                    MIN_DELAY[edge.id].append(-1)
                    continue
            min_delay = find_min_delay(output_addr_time_patterns, 
                                       input_addr_time_patterns, 
                                       k + 1, 
                                       max_channel_width,
                                       db.global_constraint.max_latency)
            MIN_DELAY[edge.id].append(min_delay)
            if (min_delay > 0):
                # TODO: If fire times > 1, what to be adjusted? 
                model.Add(F[edge.source_node][0] + min_delay < F[edge.target_node][0]).OnlyEnforceIf(K_VECS[edge.id][k])
            else:
                model.Add(K_VECS[edge.id][k] == 0)
        model.Add(sum(K_VECS[edge.id]) == 1)
    
    print("starting to optimize channel and K")
    # define a sum of w1*k + w2*delay to be an optimized object
    weighted_terms = []   
    total_delay = []
    for edge_id in K_VARS:
        index = model.NewIntVar(0, len(MIN_DELAY[edge_id]) - 1, f'index_{edge_id}')
        model.Add(index == K_VARS[edge_id] - 1)

        # a variable to hold the delay value selected
        delay_var = model.NewIntVar(0, max(MIN_DELAY[edge_id]), f'delay_{edge_id}')
        model.AddElement(index, MIN_DELAY[edge_id], delay_var)
        total_delay.append(delay_var)

        # For K*delay: create intVar for the product
        # weighted = model.NewIntVar(0, 10000, f'weighted_{edge_id}')
        # model.AddMultiplicationEquality(weighted, [K_VARS[edge_id], delay_var])
        # weighted_terms.append(weighted)
        weighted_terms.append(delay_var + 2*K_VARS[edge_id])


    # objective: minimize the weighted channel width k*delay
    obj = model.NewIntVar(0, 100000, "obj")
    model.Add(obj == sum(weighted_terms))
    #model.Add(obj == sum([K_VARS[edge_id]*DISTANCE[edge_id] for edge_id in DISTANCE]))
    #model.Add(obj == sum(total_delay))
    model.Minimize(obj)

    # solve the optimization model
    solver = cp_model.CpSolver()
    status = solver.Solve(model)
    if status != cp_model.OPTIMAL:
        logging.error("Cannot find a solution for optimizing channel width")
        sys.exit(1)
   
    K = { edge_id: [solver.Value(K_VARS[edge_id]), MIN_DELAY[edge_id][solver.Value(K_VARS[edge_id])-1]] for edge_id in K_VARS}
    # find the max_latency
    max_latency = 0
    for node in db.app_graph.nodes:
        if max_latency < solver.Value(END_TIME[node.id]):
            max_latency = solver.Value(END_TIME[node.id])
    print("max_latency=", max_latency)
    db.synthesized_information.max_latency = 2 * max_latency # relaxation
    return K

'''
T0 is the start time of each chunk
T1 is the end time of each chunk
buffer_capacity is the capacity of the buffer
return the assigned address for each chunk
'''
def equitable_address_assignment(T0, T1, buffer_capacity) -> list:
    # build a conflict graph for each chunk
    conflict_graph = {}
    for i in range(len(T0)):
        conflict_graph[i] = []
        for j in range(len(T0)):
            if i != j:
                x = T1[j] - T0[i]
                y = T1[i] - T0[j]
                if x > 0 and y > 0:
                    conflict_graph[i].append(j)
    # create a list of index of elements in T0 sorted by their value
    sorted_index = sorted(range(len(T0)), key=lambda k: T0[k])

    assigned_address = [-1 for i in range(len(T0))]
    for i in sorted_index:
        # order the address in buffer based on its assigned chunk count
        address_count = [0 for i in range(buffer_capacity)]
        for j in range(len(assigned_address)):
            if assigned_address[j] >= 0:
                address_count[assigned_address[j]] += 1
        # create a list of available address based on the address_count, smaller address_count has higher priority
        available_address = sorted(range(len(address_count)), key=lambda k: address_count[k])
        # assign the chunk to the first available address, if it has conflict with other chunks, assign to the next available address, and so on
        for j in range(len(available_address)):
            candidate_address = available_address[j]
            # find all chunks that have been assigned to address candidate_address
            assigned_chunks = []
            for k in range(len(assigned_address)):
                if assigned_address[k] == candidate_address:
                    assigned_chunks.append(k)
            # check if the current chunk has conflict with any of the assigned chunks
            has_conflict = False
            for k in range(len(assigned_chunks)):
                if assigned_chunks[k] in conflict_graph[i]:
                    has_conflict = True
                    break
            if not has_conflict:
                assigned_address[i] = candidate_address
                break
    
    # check if all chunks have been assigned an address
    for i in range(len(assigned_address)):
        if assigned_address[i] < 0:
            logging.error("Error: equitable address assignment failed!")
            sys.exit(1)
    return assigned_address

'''
This function is not applicable for the start node
return fire_time, end_time, K, OB, IB, T0, T1, T2, T3 
'''
def optimize_node_schedule(db: ds.DataBase, node_id: str, fire_times: dict, channel_bandwidth_and_delay: dict) -> dict:
    MAX_BUFFER_SIZE = 1000
    MAX_OBJ = 10000
    model = cp_model.CpModel()
    
    # get ID of all edges connected to the input of this node 
    edge_ids = [edge.id for edge in db.app_graph.edges if edge.target_node == node_id]

    fire_time = model.NewIntVar(0, db.synthesized_information.max_latency, f"fire_{node_id}")
    end_time = model.NewIntVar(0, db.synthesized_information.max_latency, f"end_{node_id}")
    for node in db.app_graph.nodes:
        if node.id == node_id:
            model.Add(end_time == fire_time + node.execution_time)
            break
       
    # create variables for input and output buffer size
    IB = {}
    OB = {}
    for edge in db.app_graph.edges:
        if edge.id in edge_ids:
            input_addr_time_patterns = translate_addr_time_patterns(db, edge.target_node, 
                                                                    edge.target_port, 'in')
            min_input_buffer_size = find_parallelization_degree(input_addr_time_patterns) 
            IB[edge.target_port] = model.NewIntVar(min_input_buffer_size, MAX_BUFFER_SIZE, "IB_"+edge.target_port)
        
            output_addr_time_patterns = translate_addr_time_patterns(db, edge.source_node, 
                                                                     edge.source_port, 'out')
            min_output_buffer_size = find_parallelization_degree(output_addr_time_patterns) 
            OB[edge.source_port] = model.NewIntVar(min_output_buffer_size, MAX_BUFFER_SIZE, "OB_"+edge.source_port)
 
    # add constraints for scheduling 
    K = {}
    T0_ALL = {}
    T1_ALL = {}
    T2_ALL = {}
    T3_ALL = {}
    for edge in db.app_graph.edges:
        if edge.id in edge_ids:
            input_addr_time_patterns = translate_addr_time_patterns(db, edge.target_node, 
                                                                    edge.target_port, 'in')
            output_addr_time_patterns = translate_addr_time_patterns(db, edge.source_node, 
                                                                     edge.source_port, 'out')
            for route in db.synthesized_information.routing_paths:
                if route.app_edge_id == edge.id:
                    wire_delay = route.delay
                    break
            else:
                logging.error("Cannot find an edge in the routing paths")
                sys.exit(1)
               
            # add constraints for channel width size and fire time
            K[edge.id] = channel_bandwidth_and_delay[edge.id][0]
            total_delay = channel_bandwidth_and_delay[edge.id][1] + wire_delay
            model.Add(fire_time >= fire_times[edge.source_node] + total_delay)
            
            T0_ALL[edge.id] = {}
            T1_ALL[edge.id] = {}
            T2_ALL[edge.id] = {}
            T3_ALL[edge.id] = {}
            D01 = [model.NewIntVar(1, db.synthesized_information.max_latency, f"D01_{i}")
                   for i in range(edge.token_size)]
            D23 = [model.NewIntVar(1, db.synthesized_information.max_latency, f"D23_{i}")
                   for i in range(edge.token_size)]
        
            # for each chunk, compute its address in source and target node
            for i in range(edge.token_size):
                try:
                    source_addr_time = output_addr_time_patterns[i]
                    target_addr_time = input_addr_time_patterns[i]
                except KeyError:
                    logging.error("Fail to translate addresses")
                    sys.exit(1)

                # create model variables like this can help reducing the compute time 
                T0_ALL[edge.id][i] = model.NewIntVar(source_addr_time, db.synthesized_information.max_latency, f"T0_{i}")
                T1_ALL[edge.id][i] = model.NewIntVar(source_addr_time, db.synthesized_information.max_latency, f"T1_{i}")
                T2_ALL[edge.id][i] = model.NewIntVar(source_addr_time, db.synthesized_information.max_latency, f"T2_{i}")
                T3_ALL[edge.id][i] = model.NewIntVar(target_addr_time, db.synthesized_information.max_latency, f"T3_{i}")

                # add scheduling constraints
                model.Add(T0_ALL[edge.id][i] == fire_times[edge.source_node] + source_addr_time)
                model.Add(T2_ALL[edge.id][i] == T1_ALL[edge.id][i] + wire_delay)
                model.Add(T3_ALL[edge.id][i] == fire_time + target_addr_time)
                model.Add(D01[i] == T1_ALL[edge.id][i] - T0_ALL[edge.id][i])
                model.Add(D23[i] == T3_ALL[edge.id][i] - T2_ALL[edge.id][i])   

            # add buffer constraints
            INTERVAL0 = [model.NewIntervalVar(T0_ALL[edge.id][i], D01[i], T1_ALL[edge.id][i], f"interval0[{i}]") for i in range(edge.token_size)]
            INTERVAL1 = [model.NewIntervalVar(T2_ALL[edge.id][i], D23[i], T3_ALL[edge.id][i], f"interval1[{i}]") for i in range(edge.token_size)]
            model.AddCumulative(INTERVAL0, [1] * edge.token_size, OB[edge.source_port])
            model.AddCumulative(INTERVAL1, [1] * edge.token_size, IB[edge.target_port])

            # add constraint for channel width
            INTERVAL2 = [model.NewIntervalVar(T1_ALL[edge.id][i], 1, T1_ALL[edge.id][i] + 1, f"interval2[{i}]") for i in range(edge.token_size)]       
            model.AddCumulative(INTERVAL2, [1] * edge.token_size, K[edge.id])

    # add objective
    OBJ = model.NewIntVar(len(IB) + len(OB), MAX_OBJ, "OBJ")
    model.Add(OBJ == sum([IB[port] for port in IB]) + sum([OB[port] for port in OB]))
    model.Minimize(OBJ)

    # solving the model 
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 2 * 60.0 # 2 minutes 
    #solver.parameters.log_search_progress = True # TODO: To be commented out
    status = solver.Solve(model)
    if status != cp_model.OPTIMAL and status != cp_model.FEASIBLE:   
        logging.error("Cannot find a solution for scheduling and optimizing buffer size")
        sys.exit(1)
    
    # formatting output 
    output_fire_time = solver.Value(fire_time)    
    output_end_time = solver.Value(end_time)    
    output_IB = {port:solver.Value(IB[port]) for port in IB}
    output_OB = {port:solver.Value(OB[port]) for port in OB}
    output_K = K    
    output_T0_ALL = {}
    output_T1_ALL = {}
    output_T2_ALL = {}
    output_T3_ALL = {}
    for edge in db.app_graph.edges:
        if edge.id in edge_ids:
            output_T0_ALL[edge.id] = {i:solver.Value(T0_ALL[edge.id][i]) for i in range(edge.token_size)}
            output_T1_ALL[edge.id] = {i:solver.Value(T1_ALL[edge.id][i]) for i in range(edge.token_size)}
            output_T2_ALL[edge.id] = {i:solver.Value(T2_ALL[edge.id][i]) for i in range(edge.token_size)}
            output_T3_ALL[edge.id] = {i:solver.Value(T3_ALL[edge.id][i]) for i in range(edge.token_size)}
    
    output = {
        "F": output_fire_time,
        "END_TIME": output_end_time,
        "IB": output_IB,
        "OB": output_OB,
        "K": output_K,
        "T0": output_T0_ALL,
        "T1": output_T1_ALL,
        "T2": output_T2_ALL,
        "T3": output_T3_ALL
    }
    return output


def scheduling(db: ds.DataBase, channel_bandwidth_and_delay: dict) -> cp_model.CpModel:
    for node in db.app_graph.nodes:
        if node.repetition > 1:
            logging.error("glic does not fully support firing a node more than once")
            sys.exit(1)
        
    # create int variable for the fire time of each node
    F = {}
    END_TIME = {}
    IB = {}
    OB = {}
    K = {}
    T0_ALL = {}
    T1_ALL = {}
    T2_ALL = {}
    T3_ALL = {}
    
    # initialise with the start nodes 
    all_nodes = []
    for node in db.app_graph.nodes:
        all_nodes.append(node.id)
        if len(node.input_ports) == 0:
            F[node.id] = 0
            END_TIME[node.id] = node.execution_time
    
    # go through each node that the nodes fire to it have determined the fire times
    while (1):
        no_execute = True
        for node in db.app_graph.nodes:
            if node.id not in F:
                all_input_sources = [edge.source_node for edge in db.app_graph.edges if edge.target_node == node.id]
                all_input_sources_done = [(source in F) for source in all_input_sources]
                if all(all_input_sources_done):
                    print(f"scheduling {node.id}")
                    node_scheduling = optimize_node_schedule(db, node.id, F, channel_bandwidth_and_delay) 
                    no_execute = False 
                    F[node.id] = node_scheduling["F"]
                    END_TIME[node.id] = node_scheduling["END_TIME"]
                    IB = merge_two_dicts(IB, node_scheduling["IB"])
                    OB = merge_two_dicts(OB, node_scheduling["OB"])
                    K = merge_two_dicts(K, node_scheduling["K"])
                    T0_ALL = merge_two_dicts(T0_ALL, node_scheduling["T0"])
                    T1_ALL = merge_two_dicts(T1_ALL, node_scheduling["T1"])
                    T2_ALL = merge_two_dicts(T2_ALL, node_scheduling["T2"])
                    T3_ALL = merge_two_dicts(T3_ALL, node_scheduling["T3"])
        all_nodes_done = [(node in F) for node in all_nodes]
        if all(all_nodes_done):
            break
        
        # report error if there's no scheduling in an iteration
        if no_execute: 
            logging.error("Cannot do glic scheduling for a node")
            sys.exit(1)
   
    # TODO: add global timing verification here to check if our solution meets the global constraints

    # We can add post-optimisation stage later if the solution is not optimal enough
    print("complete schduling all nodes ")
   
    # print all var in T0_ALL, T1_ALL, T2_ALL, T3_ALL
    for edge in db.app_graph.edges:
        print(f"edge = {edge.id}")
        for i in range(edge.token_size):
            print("T0=", T0_ALL[edge.id][i], "T1=", T1_ALL[edge.id][i], "T2=", T2_ALL[edge.id][i], "T3=", T3_ALL[edge.id][i])

    for node in db.app_graph.nodes:
        db.synthesized_information.node_fire_times[node.id] = F[node.id]
    for edge in db.app_graph.edges:
        # TODO: In case of fire time >1, to figure out when each transporter fires, we should use the min value of all in T1
        db.synthesized_information.node_fire_times["transporter_"+edge.id] = 0
        db.synthesized_information.channel_width["transporter_"+edge.id] = K[edge.id]
    
    # assigning buffer size 
    for node in db.app_graph.nodes:
        db.synthesized_information.input_buffer_size[node.id] = 0
        db.synthesized_information.output_buffer_size[node.id] = 0
        for input_port in node.input_ports:
            db.synthesized_information.input_buffer_size[node.id] += IB[input_port.id]
        for output_port in node.output_ports:
            db.synthesized_information.output_buffer_size[node.id] += OB[output_port.id]

    # assigning translation tables 
    for node in db.app_graph.nodes:
        out_offset = 0 # edge offset
        for output_port in node.output_ports:
            for edge in db.app_graph.edges:
                # find the edge that use this port as source port
                if edge.source_node == node.id and edge.source_port == output_port.id:
                    T0 = [T0_ALL[edge.id][x] for x in T0_ALL[edge.id]]
                    T1 = [T1_ALL[edge.id][x] for x in T1_ALL[edge.id]]
                    buffer_capacity = OB[output_port.id]
                    assigned_address = equitable_address_assignment(T0, T1, buffer_capacity)
                    # add edge offset to the address list 
                    for i in range(len(assigned_address)):
                        assigned_address[i] += out_offset
                    out_offset += buffer_capacity

                    chunk_address_assignment = ds.ChunkAddressAssignment()
                    chunk_address_assignment.app_node_id = node.id
                    chunk_address_assignment.port_id = output_port.id

                    for i in range(len(T0)):
                        chunk_address_assignment.address_assignment[translate_source_addr(db.app_graph, edge.source_node, edge.source_port, i)] = assigned_address[i]
                    
                    db.synthesized_information.chunk_address_assignments.append(chunk_address_assignment)
        
        in_offset = 0 # edge offset 
        for input_port in node.input_ports:
            for edge in db.app_graph.edges:
                # find the edge that use this port as target port
                if edge.target_node == node.id and edge.target_port == input_port.id:
                    T0 = [T2_ALL[edge.id][x] for x in T2_ALL[edge.id]]
                    T1 = [T3_ALL[edge.id][x] for x in T3_ALL[edge.id]]
                    buffer_capacity = IB[input_port.id]
                    assigned_address = equitable_address_assignment(T0, T1, buffer_capacity)
                    # add edge offset to the address list 
                    for i in range(len(assigned_address)):
                        assigned_address[i] += in_offset
                    in_offset += buffer_capacity

                    chunk_address_assignment = ds.ChunkAddressAssignment()
                    chunk_address_assignment.app_node_id = node.id
                    chunk_address_assignment.port_id = input_port.id

                    for i in range(len(T0)):
                        chunk_address_assignment.address_assignment[translate_target_addr(db.app_graph, edge.target_node, edge.target_port, i)] = assigned_address[i]
                    
                    db.synthesized_information.chunk_address_assignments.append(chunk_address_assignment)
   
    # assigning transporter instructions
    for edge in db.app_graph.edges:
        for x in T1_ALL[edge.id]:
            time = T1_ALL[edge.id][x] - db.synthesized_information.node_fire_times["transporter_"+edge.id]
            #print("time=", time, "abs_time=", solver.Value(x), "fire_time=", db.synthesized_information.node_fire_times["transporter_"+edge.id])
            virtual_source_address = translate_source_addr(db.app_graph, edge.source_node, edge.source_port, x)
            virtual_target_address = translate_target_addr(db.app_graph, edge.target_node, edge.target_port, x)
            for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
                if chunk_address_assignment.app_node_id == edge.source_node and chunk_address_assignment.port_id == edge.source_port:
                    if virtual_source_address in chunk_address_assignment.address_assignment:
                        source_address = chunk_address_assignment.address_assignment[virtual_source_address]
                        break
                    else:
                        logging.error("Error: source address not found in node "+edge.source_node)
                        sys.exit(1)
            for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
                if chunk_address_assignment.app_node_id == edge.target_node and chunk_address_assignment.port_id == edge.target_port:
                    if virtual_target_address in chunk_address_assignment.address_assignment:
                        target_address = chunk_address_assignment.address_assignment[virtual_target_address]
                        break
                    else:
                        logging.error("Error: target address not found in node "+edge.target_node)
                        sys.exit(1)
            db.synthesized_information.transport_tables[edge.id].entries.append(ds.TransportTableEntry(source_address=source_address, target_address=target_address, time=time))


'''
def create_data_vector(length) -> tuple:
    data_production_vector = [0 for i in range(length)]
    data_consumption_vector = [0 for i in range(length)]
'''
def find_repetition(db: ds.DataBase):
    for node in db.app_graph.nodes:
        node.repetition = 1
        for alimp_binding in db.synthesized_information.alimp_bindings:
            if alimp_binding.app_node_id == node.id:
                node.execution_time = alimp_binding.alimp_instance.latency
                break

def run(db: ds.DataBase):
    logging.info("Start: glic")
    find_repetition(db)
    print("<< optimizing channel width and delay >>")
    channel_width_and_delay = optimize_channel_width(db)
    print("channel bandwidth and delay =", channel_width_and_delay)
    print("<< scheduling >>")
    scheduling(db, channel_width_and_delay)

    # print node_fire_times, transporter_fire_times, channel_width, input_buffer_size, output buffer_size in db.synthesized_information
    print("node_fire_times=", db.synthesized_information.node_fire_times)
    print("channel_width=", db.synthesized_information.channel_width)
    print("input_buffer_size=", db.synthesized_information.input_buffer_size)
    print("output_buffer_size=", db.synthesized_information.output_buffer_size)

    ## print chunk_address_assignment in db.synthesized_information
    #for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
    #    print(chunk_address_assignment.app_node_id, chunk_address_assignment.port_id, chunk_address_assignment.address_assignment)
    logging.info("Finish: glic")

if __name__ == "__main__":
    assignment = equitable_address_assignment(
                [2, 2, 2, 2, 2, 2, 4, 7, 3, 10, 8, 6, 5, 9, 7, 10], 
                [3, 3, 4, 4, 14, 8, 5, 8, 13, 11, 10, 11, 12, 12, 10, 13],
                7)
    print(assignment)
