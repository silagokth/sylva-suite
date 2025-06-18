import os
import sys
import logging

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import lib.proto.data_structure_pb2 as ds

from ortools.sat.python import cp_model

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

def find_min_delay(src_addr_pattern: dict, dest_addr_pattern: dict, channel_width: int, max_delay: int) -> int:
    # find the common address of both src_addr_pattern and dest_addr_pattern and store in a set
    common_addr = set()
    for addr in src_addr_pattern:
        if addr in dest_addr_pattern:
            common_addr.add(addr)
    
    # Use OR-tools to find the minimum delay
    model = cp_model.CpModel()
    # create variables
    delay = model.NewIntVar(2, max_delay, "delay")
    # create TRANSPORTER VARIABLES for READ for each address
    tr_read = {}
    for addr in common_addr:
        tr_read[addr] = model.NewIntVar(0, max_delay, f"tr_read_{addr}")
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
    status = solver.Solve(model)
    if status == cp_model.OPTIMAL:
        print("DELAY=", solver.Value(delay))
        print("SRC=", [src_addr_pattern[addr] for addr in common_addr])
        print("DEST=", [dest_addr_pattern[addr] for addr in common_addr])
        print("TR_READ=", [solver.Value(tr_read[addr]) for addr in common_addr])
        return solver.Value(delay)
    else:
        logging.error("Cannot find a delay value for the provided address time patterns")
        sys.exit(1)


def find_parallelization_degree(addr_pattern: dict) -> int:
    # return the max count of the same value in addr_pattern
    max_count = 0
    for addr in addr_pattern:
        count = sum([1 for x in addr_pattern if addr_pattern[addr] == addr_pattern[x]])
        if count > max_count:
            max_count = count
    return max_count


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
    # for each edge, create a variable for the fire time of the transporter
    for edge in db.app_graph.edges:
        F["transporter_"+edge.source_port + "_" +
            edge.target_port] = [model.NewIntVar(0, db.global_constraint.max_latency, "transporter_"+edge.source_port + "_" +
                                                 edge.target_port+"_0")]
    
    K_VARS = {}
    K_VECS = {}
    DISTANCE = {}
    MIN_DELAY = {}
    # for each edge, find the min delay for each channel width
    # At this stage, we don't have to consider wire delay
    for edge in db.app_graph.edges:
        for node in db.synthesized_information.alimp_bindings:
            if node.app_node_id == edge.source_node:
                output_addr_time_patterns = node.alimp_instance.output_addr_time_patterns
                break
        else:
            logging.error("Cannot find the address time patterns in Alimp")
            sys.exit(1)

        for node in db.synthesized_information.alimp_bindings:
            if node.app_node_id == edge.target_node:
                input_addr_time_patterns = node.alimp_instance.input_addr_time_patterns
                break
        else:
            logging.error("Cannot find the address time patterns in Alimp")
            sys.exit(1)

        for route in db.synthesized_information.routing_paths:
            if route.app_edge_id == edge.id:
                DISTANCE[edge.id] = len(route.path)
                break
        else:
            logging.error("Cannot find an edge in the routing paths")
            sys.exit(1)
               
        max_channel_width = max(find_parallelization_degree(pair_int_int_to_dict(output_addr_time_patterns)), 
                                find_parallelization_degree(pair_int_int_to_dict(input_addr_time_patterns)))
        K_VARS[edge.id] = model.NewIntVar(1, max_channel_width, "K_VARS_"+edge.id)
        K_VECS[edge.id] = []
        MIN_DELAY[edge.id] = []
        for k in range (max_channel_width):
            K_VECS[edge.id].append(model.NewBoolVar("K_VECS_"+edge.id+"_"+str(k)))
            model.Add(K_VARS[edge.id] == k + 1).OnlyEnforceIf(K_VECS[edge.id][k])
            min_delay = find_min_delay(pair_int_int_to_dict(output_addr_time_patterns), 
                                       pair_int_int_to_dict(input_addr_time_patterns), 
                                       k + 1, 
                                       db.global_constraint.max_latency)
            # TODO: If fire times > 1, what to be adjusted? 
            model.Add(F[edge.source_node][0] + min_delay < F[edge.target_node][0]).OnlyEnforceIf(K_VECS[edge.id][k])
            MIN_DELAY[edge.id].append(min_delay)
        model.Add(sum(K_VECS[edge.id]) == 1)
    
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
    obj = model.NewIntVar(0, 10000, "obj")
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
    db.synthesized_information.max_latency = max_latency
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
                if T0[i] < T0[j] and T0[j] < T1[i]:
                    conflict_graph[i].append(j)
                elif T0[i] < T1[j] and T1[j] < T1[i]:
                    conflict_graph[i].append(j)
                elif T0[j] < T0[i] and T1[i] < T1[j]:
                    conflict_graph[i].append(j)
                elif T0[j] < T0[i] and T0[i] < T1[j]:
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


def optimize_buffer_size(db: ds.DataBase, channel_bandwidth_and_delay: dict) -> cp_model.CpModel:

    print("channel bandwidth and delay=", channel_bandwidth_and_delay)
    MAX_LATENCY = db.synthesized_information.max_latency+10
    MAX_BUFFER_SIZE = 1000
    MAX_OBJ = 10000
    model = cp_model.CpModel()

    # create int variable for the fire time of each node
    F = {}
    all_nodes = []
    END_TIME = {}
    for node in db.app_graph.nodes:
        all_nodes.append(node.id)
        F[node.id] = [model.NewIntVar(0, MAX_LATENCY, node.id+"_"+str(i))
                      for i in range(node.repetition)]
        END_TIME[node.id] = [model.NewIntVar(0, db.global_constraint.max_latency, f"{END_TIME}_{node.id}_{i}")
                             for i in range(node.repetition)]
        for i in range(node.repetition):
            if i > 0:
                model.Add(F[node.id][i] > END_TIME[node.id][i-1]) 
            model.Add(END_TIME[node.id][i] == F[node.id][i]+node.execution_time)
    # the minimal of F should be 0
    model.AddMinEquality(0, [F[node.id][0] for node in db.app_graph.nodes])
    # for each edge, create a variable for the fire time of the tranporter
    for edge in db.app_graph.edges:
        F["transporter_"+edge.id] = [model.NewIntVar(0, MAX_LATENCY, "transporter_"+edge.id)]

    # create variable for input and output buffer size
    IB = {}
    OB = {}
    for edge in db.app_graph.edges:
        IB[edge.target_port] = model.NewIntVar(0, MAX_BUFFER_SIZE, "IB_"+edge.target_port)
        OB[edge.source_port] = model.NewIntVar(0, MAX_BUFFER_SIZE, "OB_"+edge.source_port)

    TR = {}
    T0_ALL = {}
    T1_ALL = {}
    T2_ALL = {}
    T3_ALL = {}
    K = {}
    # add constraints for token chunk of each edge
    for edge in db.app_graph.edges:
        for node in db.synthesized_information.alimp_bindings:
            if node.app_node_id == edge.source_node:
                output_addr_time_patterns = node.alimp_instance.output_addr_time_patterns
                break
        else:
            logging.error("Cannot find the address time patterns in Alimp")
            sys.exit(1)

        for node in db.synthesized_information.alimp_bindings:
            if node.app_node_id == edge.target_node:
                input_addr_time_patterns = node.alimp_instance.input_addr_time_patterns
                break
        else:
            logging.error("Cannot find the address time patterns in Alimp")
            sys.exit(1)

        for route in db.synthesized_information.routing_paths:
            if route.app_edge_id == edge.id:
                wire_delay = route.delay
                break
        else:
            logging.error("Cannot find an edge in the routing paths")
            sys.exit(1)
               
        TR[edge.id] = {}
        T0_ALL[edge.id] = {}
        T1_ALL[edge.id] = {}
        T2_ALL[edge.id] = {}
        T3_ALL[edge.id] = {}

        T0 = [model.NewIntVar(0, MAX_LATENCY, "T0_"+str(i))
              for i in range(edge.token_size)]
        T1 = [model.NewIntVar(0, MAX_LATENCY, "T1_"+str(i))
              for i in range(edge.token_size)]
        T2 = [model.NewIntVar(0, MAX_LATENCY, "T2_"+str(i))
              for i in range(edge.token_size)]
        T3 = [model.NewIntVar(0, MAX_LATENCY, "T3_"+str(i))
              for i in range(edge.token_size)]
        D01 = [model.NewIntVar(1, MAX_LATENCY, "D01_"+str(i))
               for i in range(edge.token_size)]
        D23 = [model.NewIntVar(1, MAX_LATENCY, "D23_"+str(i))
               for i in range(edge.token_size)]
        # create variable for channel width
        K[edge.id] = channel_bandwidth_and_delay[edge.id][0]
        total_delay = channel_bandwidth_and_delay[edge.id][1] + wire_delay
        model.Add(F[edge.target_node][0] >= F[edge.source_node][0] + total_delay)
        # for each chunk, compute its address in source and target node
        idx = 0
        for addr in range(edge.token_size):
            T0_ALL[edge.id][addr] = T0[idx]
            T1_ALL[edge.id][addr] = T1[idx]
            T2_ALL[edge.id][addr] = T2[idx]
            T3_ALL[edge.id][addr] = T3[idx]

            # translate address in source node
            source_addr = translate_source_addr(db.app_graph,
                                                edge.source_node, edge.source_port, addr)
            # translate address in target node
            target_addr = translate_target_addr(db.app_graph,
                                                edge.target_node, edge.target_port, addr)
            
            # source address time
            source_addr_time = -1
            for addr_time_pattern in output_addr_time_patterns:
                if addr_time_pattern.key == source_addr:
                    source_addr_time = addr_time_pattern.value
                    break
            # target address time
            target_addr_time = -1
            for addr_time_pattern in input_addr_time_patterns:
                if addr_time_pattern.key == target_addr:
                    target_addr_time = addr_time_pattern.value
                    break
            #print("source_addr_time=", source_addr_time, "target_addr_time=", target_addr_time, "edge=", edge.id)
            #print("addr=", addr, "source_addr=", source_addr, "target_addr=", target_addr)
            if source_addr_time < 0 or target_addr_time < 0:
                logging.error("Fail to translate addresses")
                sys.exit(1)

            # create variable for transporter read and write
            TR[edge.id][addr] = model.NewIntVar(0, MAX_LATENCY, "TR_"+edge.id+"_"+str(addr))

            # add constraints
            model.Add(T0[idx] == F[edge.source_node][0] + source_addr_time)
            model.Add(T1[idx] == TR[edge.id][addr])
            model.Add(T2[idx] == T1[idx] + wire_delay)
            model.Add(T3[idx] == F[edge.target_node][0] + target_addr_time)
            model.Add(D01[idx] == T1[idx] - T0[idx])
            model.Add(D23[idx] == T3[idx] - T2[idx])
            model.Add(D01[idx] > 0) 
            model.Add(D23[idx] > 0)
            idx += 1

        INTERVAL0 = [model.NewIntervalVar(T0[i], D01[i], T1[i], f"interval0[{i}]") for i in range(edge.token_size)]
        INTERVAL1 = [model.NewIntervalVar(T2[i], D23[i], T3[i], f"interval1[{i}]") for i in range(edge.token_size)]
        model.AddCumulative(INTERVAL0, [1 for i in range(edge.token_size)], OB[edge.source_port])
        model.AddCumulative(INTERVAL1, [1 for i in range(edge.token_size)], IB[edge.target_port])

        # add constraint for channel width
        # the count of any value in T1 should be less than K[edge.source_port+"_"+edge.target_port]
        T1_1 = [model.NewIntVar(0, MAX_LATENCY, "T1_1_"+str(i)) for i in range(edge.token_size)]
        INTERVAL2 = [model.NewIntervalVar(T1[i], 1, T1_1[i], f"interval2[{i}]") for i in range(edge.token_size)]       
        model.AddCumulative(INTERVAL2, [1 for i in range(edge.token_size)], K[edge.id])

    # add objective
    OBJ = model.NewIntVar(0, MAX_OBJ, "OBJ")
    model.Add(OBJ == sum([IB[port] for port in IB]) + sum([OB[port] for port in OB]))
    model.Minimize(OBJ)

    solver = cp_model.CpSolver()
    status = solver.Solve(model)
    if status != cp_model.OPTIMAL:
        logging.error("Cannot find a solution for scheduling and optimizing buffer size")
        sys.exit(1)
        
    print("OBJ=", solver.Value(OBJ))
    print("IB AND OB=")
    for var in IB:
        print(var, solver.Value(IB[var]))
    for var in OB:
        print(var, solver.Value(OB[var]))
    
    # print all var in T0_ALL, T1_ALL, T2_ALL, T3_ALL
    for edge in db.app_graph.edges:
        print(edge.id)
        for i in range(edge.token_size):
            print("T0=", solver.Value(T0_ALL[edge.id][i]), "T1=", solver.Value(T1_ALL[edge.id][i]), "T2=", solver.Value(T2_ALL[edge.id][i]), "T3=", solver.Value(T3_ALL[edge.id][i]))

    for node in db.app_graph.nodes:
        db.synthesized_information.node_fire_times[node.id] = solver.Value(F[node.id][0])
    for edge in db.app_graph.edges:
        db.synthesized_information.node_fire_times["transporter_"+edge.id] = solver.Value(F["transporter_"+edge.id][0])
        db.synthesized_information.channel_width["transporter_"+edge.id] = solver.Value(K[edge.id])
    
    # assigning buffer size 
    for node in db.app_graph.nodes:
        db.synthesized_information.input_buffer_size[node.id] = 0
        db.synthesized_information.output_buffer_size[node.id] = 0
        for input_port in node.input_ports:
            db.synthesized_information.input_buffer_size[node.id] += solver.Value(IB[input_port.id])
        for output_port in node.output_ports:
            db.synthesized_information.output_buffer_size[node.id] += solver.Value(OB[output_port.id])

    # assigning translation tables 
    for node in db.app_graph.nodes:
        out_offset = 0 # edge offset
        for output_port in node.output_ports:
            for edge in db.app_graph.edges:
                # find the edge that use this port as source port
                if edge.source_node == node.id and edge.source_port == output_port.id:
                    #T0 = [solver.Value(T0_ALL[edge.id][i]) for i,_ in enumerate(T0_ALL[edge.id])]
                    T0 = [solver.Value(T0_ALL[edge.id][x]) for x in T0_ALL[edge.id]]
                    T1 = [solver.Value(T1_ALL[edge.id][x]) for x in T1_ALL[edge.id]]
                    buffer_capacity = solver.Value(OB[output_port.id])
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
                    T0 = [solver.Value(T2_ALL[edge.id][x]) for x in T2_ALL[edge.id]]
                    T1 = [solver.Value(T3_ALL[edge.id][x]) for x in T3_ALL[edge.id]]
                    buffer_capacity = solver.Value(IB[input_port.id])
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
            time = solver.Value(T1_ALL[edge.id][x]) - db.synthesized_information.node_fire_times["transporter_"+edge.id]
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
    find_repetition(db)
    channel_width_and_delay = optimize_channel_width(db)
    optimize_buffer_size(db, channel_width_and_delay)

    # print node_fire_times, transporter_fire_times, channel_width, input_buffer_size, output buffer_size in db.synthesized_information
    print("node_fire_times=", db.synthesized_information.node_fire_times)
    print("channel_width=", db.synthesized_information.channel_width)
    print("input_buffer_size=", db.synthesized_information.input_buffer_size)
    print("output_buffer_size=", db.synthesized_information.output_buffer_size)

    # print chunk_address_assignment in db.synthesized_information
    for chunk_address_assignment in db.synthesized_information.chunk_address_assignments:
        print(chunk_address_assignment.app_node_id, chunk_address_assignment.port_id, chunk_address_assignment.address_assignment)

if __name__ == "__main__":
    assignment = equitable_address_assignment(
                [2, 2, 2, 2, 2, 2, 4, 7, 3, 10, 8, 6, 5, 9, 7, 10], 
                [3, 3, 4, 4, 14, 8, 5, 8, 13, 11, 10, 11, 12, 12, 10, 13],
                7)
    print(assignment)
