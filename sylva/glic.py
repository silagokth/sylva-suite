import os
import sys

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import data_structure_pb2 as ds

from ortools.sat.python import cp_model


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


def create_cp_model(app_graph: ds.AppGraph) -> cp_model.CpModel:
    model = cp_model.CpModel()
    MAX_TIME = 100
    MAX_BUFFER_SIZE = 32
    MAX_CHANNEL_NUMBER = 4
    MAX_OBJ = 100
    COEFFICIENT_BUFFER = 1
    COEFFICIENT_CHANNEL = 2

    CONSTRAINT_LATENCY = 20
    CONSTRAINT_PERIOD = 5
    # create int variable for the fire time of each node
    F = {}
    all_nodes = []
    END_TIME = {}
    for node in app_graph.nodes:
        all_nodes.append(node.id)
        F[node.id] = [model.NewIntVar(0, MAX_TIME, node.id+"_"+str(i))
                      for i in range(node.repetition)]
        END_TIME[node.id] = model.NewIntVar(0, MAX_TIME, "END_TIME_"+node.id)
        model.Add(END_TIME[node.id] == F[node.id]
                  [node.repetition-1]+node.execution_time)
    # for each edge, create a variable for the fire time of the tranporter
    for edge in app_graph.edges:
        F["transporter_"+edge.source_port + "_" +
            edge.target_port] = [model.NewIntVar(0, MAX_TIME, "transporter_"+edge.source_port + "_" +
                                                 edge.target_port+"_0")]

    # create variable for input and output buffer size
    IB = {}
    OB = {}
    for edge in app_graph.edges:
        IB[edge.target_port] = model.NewIntVar(
            0, MAX_BUFFER_SIZE, "IB_"+edge.target_port)
        OB[edge.source_port] = model.NewIntVar(
            0, MAX_BUFFER_SIZE, "OB_"+edge.source_port)

    TR = {}
    K = {}
    # add constraints for token chunk of each edge
    for edge in app_graph.edges:
        output_addr_time_patterns = {}
        input_addr_time_patterns = {}
        for node in app_graph.nodes:
            if node.id == edge.source_node:
                for port in node.output_ports:
                    if port.id == edge.source_port:
                        output_addr_time_patterns = port.addr_time_patterns
                        break
            if node.id == edge.target_node:
                for port in node.input_ports:
                    if port.id == edge.target_port:
                        input_addr_time_patterns = port.addr_time_patterns
                        break
            if output_addr_time_patterns and input_addr_time_patterns:
                break

        TR[edge.source_port] = {}
        T0 = [model.NewIntVar(0, MAX_TIME, "T0_"+str(i))
              for i in range(edge.token_size)]
        T1 = [model.NewIntVar(0, MAX_TIME, "T1_"+str(i))
              for i in range(edge.token_size)]
        T2 = [model.NewIntVar(0, MAX_TIME, "T2_"+str(i))
              for i in range(edge.token_size)]
        T3 = [model.NewIntVar(0, MAX_TIME, "T3_"+str(i))
              for i in range(edge.token_size)]
        D01 = [model.NewIntVar(0, MAX_TIME, "D01_"+str(i))
               for i in range(edge.token_size)]
        D23 = [model.NewIntVar(0, MAX_TIME, "D23_"+str(i))
               for i in range(edge.token_size)]
        # create variable for channel width
        K[edge.source_port+"_"+edge.target_port] = model.NewIntVar(
            0, MAX_CHANNEL_NUMBER, "K_"+edge.source_port+"_"+edge.target_port)
        # for each chunk, compute its address in source and target node
        idx = 0
        for addr in range(edge.token_size):
            # translate address in source node
            source_addr = translate_source_addr(app_graph,
                                                edge.source_node, edge.source_port, addr)
            # translate address in target node
            target_addr = translate_target_addr(app_graph,
                                                edge.target_node, edge.target_port, addr)
            # source address time
            source_addr_time = -1
            for addr_time_pattern in output_addr_time_patterns:
                print(addr_time_pattern)
                if addr_time_pattern.key == source_addr:
                    source_addr_time = addr_time_pattern.value
                    break
            # target address time
            target_addr_time = -1
            for addr_time_pattern in input_addr_time_patterns:
                if addr_time_pattern.key == target_addr:
                    target_addr_time = addr_time_pattern.value
                    break
            if source_addr_time < 0 or target_addr_time < 0:
                print("Error: address translation failed!")
                sys.exit(1)

            # create variable for transporter read and write
            TR[edge.source_port][addr] = model.NewIntVar(
                0, MAX_TIME, "TR_"+edge.source_port+"_"+str(addr))

            # add constraint
            model.Add(T0[idx] == F[edge.source_node][0] + source_addr_time)
            model.Add(T1[idx] == TR[edge.source_port][addr])
            model.Add(T2[idx] == T1[idx] + edge.delay)
            model.Add(T3[idx] == F[edge.target_node][0] + target_addr_time)
            model.Add(D01[idx] == T1[idx] - T0[idx])
            model.Add(D23[idx] == T3[idx] - T2[idx])

            idx += 1

        INTERVAL0 = [model.NewIntervalVar(
            T0[i], D01[i], T1[i], f"interval0[{i}]") for i in range(edge.token_size)]
        INTERVAL1 = [model.NewIntervalVar(
            T2[i], D23[i], T3[i], f"interval1[{i}]") for i in range(edge.token_size)]
        model.AddCumulative(INTERVAL0, [1 for i in range(
            edge.token_size)], OB[edge.source_port])
        model.AddCumulative(INTERVAL1, [1 for i in range(
            edge.token_size)], IB[edge.target_port])

        # add constraint for channel width
        # the count of any value in T1 should be less than K[edge.source_port+"_"+edge.target_port]
        T1_1 = [model.NewIntVar(0, MAX_TIME, "T1_1_"+str(i)) for i in range(edge.token_size)]
        INTERVAL2 = [model.NewIntervalVar(
            T1[i], 1, T1_1[i], f"interval2[{i}]") for i in range(edge.token_size)]
        
        model.AddCumulative(INTERVAL2, [1 for i in range(
            edge.token_size)], K[edge.source_port+"_"+edge.target_port])

    # total latency
    latency = model.NewIntVar(0, MAX_TIME, "latency")
    model.AddMaxEquality(latency, [END_TIME[id]
                                   for id in all_nodes])
    model.Add(latency <= CONSTRAINT_LATENCY)

    # add objective
    OBJ = model.NewIntVar(0, MAX_OBJ, "OBJ")
    model.Add(OBJ == COEFFICIENT_BUFFER *
              (sum([IB[port] for port in IB]) + sum([OB[port] for port in OB])) + COEFFICIENT_CHANNEL * (sum([K[port] for port in K])))
    model.Minimize(OBJ)

    solver = cp_model.CpSolver()
    status = solver.Solve(model)
    if status == cp_model.OPTIMAL or status == cp_model.FEASIBLE:
        for node in g.nodes:
            print(node.id, [solver.Value(F[node.id][i])
                            for i in range(node.repetition)])
        for edge in g.edges:
            print("transporter_"+edge.source_port + "_" +
                  edge.target_port, [solver.Value(F["transporter_"+edge.source_port + "_" +
                                                    edge.target_port][i]) for i in range(1)])
        print("IB=", [solver.Value(IB[port]) for port in IB])
        print("OB=", [solver.Value(OB[port]) for port in OB])
        print("K=", [solver.Value(K[port]) for port in K])
        print("OBJ=", solver.Value(OBJ))
    else:
        print("No solution found!")


def create_data_vector(length) -> tuple:
    data_production_vector = [0 for i in range(length)]
    data_consumption_vector = [0 for i in range(length)]


if __name__ == "__main__":
    g = create_app_graph()
    print(MessageToJson(g))
    m = create_cp_model(g)
