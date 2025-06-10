import sys
import os
import lib.proto.data_structure_pb2 as ds
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse
import random
import json

def create_test_db(testcase_name:str):
    this_module = sys.modules[__name__]
    func = getattr(this_module, testcase_name)

    db = func()
    # write db.app_graph to json file
    app_graph_json = MessageToJson(db.app_graph)
    with open(os.path.join('const/app_graph.json'), 'w') as f:
        f.write(app_graph_json)
    # write db.global_constraint to json file
    global_constraint_json = MessageToJson(db.global_constraint)
    with open(os.path.join('const/global_constraint.json'), 'w') as f:
        f.write(global_constraint_json)
    # write db.alimp_lib to json file
    alimp_lib_json = MessageToJson(db.alimp_lib)
    with open(os.path.join('const/alimp_lib.json'), 'w') as f:
        f.write(alimp_lib_json)
    # write db.hyper_parameter to json file
    hyper_parameter_json = MessageToJson(db.hyper_parameter)
    with open(os.path.join('const/hyper_parameter.json'), 'w') as f:
        f.write(hyper_parameter_json)
    #TODO: write db.noc_constraint to json file

def sobel() -> ds.DataBase:
    db = ds.DataBase()
    db.global_constraint.max_energy = 100
    db.global_constraint.max_width = 100
    db.global_constraint.max_height = 100
    db.global_constraint.max_latency = 100
    db.global_constraint.max_period = 70

    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1
    db.hyper_parameter.place_reserved_routing_size = 1

    func_load_entry = ds.AlimpEntry()
    func_load_entry.func = "func_load"
    func_load_entry.instances.append(ds.AlimpInstance(width=2, height=2, energy=1, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)]))
    func_load_entry.instances.append(ds.AlimpInstance(width=4, height=4, energy=2, latency=4, output_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=0), ds.pair_int_int(key=2, value=0), ds.pair_int_int(key=3, value=0), ds.pair_int_int(key=4, value=1), ds.pair_int_int(key=5, value=1), ds.pair_int_int(key=6, value=1), ds.pair_int_int(key=7, value=1), ds.pair_int_int(key=8, value=2), ds.pair_int_int(key=9, value=2), ds.pair_int_int(key=10, value=2), ds.pair_int_int(key=11, value=2), ds.pair_int_int(key=12, value=3), ds.pair_int_int(key=13, value=3), ds.pair_int_int(key=14, value=3), ds.pair_int_int(key=15, value=3)]))
    db.alimp_lib.entries.append(func_load_entry)
    func_copy_entry = ds.AlimpEntry()
    func_copy_entry.func = "func_copy"
    func_copy_entry.instances.append(ds.AlimpInstance(width=1, height=1, energy=1, latency=17, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16), ds.pair_int_int(key=16, value=1), ds.pair_int_int(key=17, value=2), ds.pair_int_int(key=18, value=3), ds.pair_int_int(key=19, value=4), ds.pair_int_int(key=20, value=5), ds.pair_int_int(key=21, value=6), ds.pair_int_int(key=22, value=7), ds.pair_int_int(key=23, value=8), ds.pair_int_int(key=24, value=9), ds.pair_int_int(key=25, value=10), ds.pair_int_int(key=26, value=11), ds.pair_int_int(key=27, value=12), ds.pair_int_int(key=28, value=13), ds.pair_int_int(key=29, value=14), ds.pair_int_int(key=30, value=15), ds.pair_int_int(key=31, value=16)]))
    func_copy_entry.instances.append(ds.AlimpInstance(width=2, height=1, energy=2, latency=9, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=0), ds.pair_int_int(key=2, value=1), ds.pair_int_int(key=3, value=1), ds.pair_int_int(key=4, value=2), ds.pair_int_int(key=5, value=2), ds.pair_int_int(key=6, value=3), ds.pair_int_int(key=7, value=3), ds.pair_int_int(key=8, value=4), ds.pair_int_int(key=9, value=4), ds.pair_int_int(key=10, value=5), ds.pair_int_int(key=11, value=5), ds.pair_int_int(key=12, value=6), ds.pair_int_int(key=13, value=6), ds.pair_int_int(key=14, value=7), ds.pair_int_int(key=15, value=7)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=2), ds.pair_int_int(key=4, value=3), ds.pair_int_int(key=5, value=3), ds.pair_int_int(key=6, value=4), ds.pair_int_int(key=7, value=4), ds.pair_int_int(key=8, value=5), ds.pair_int_int(key=9, value=5), ds.pair_int_int(key=10, value=6), ds.pair_int_int(key=11, value=6), ds.pair_int_int(key=12, value=7), ds.pair_int_int(key=13, value=7), ds.pair_int_int(key=14, value=8), ds.pair_int_int(key=15, value=8), ds.pair_int_int(key=16, value=9), ds.pair_int_int(key=17, value=9), ds.pair_int_int(key=18, value=10), ds.pair_int_int(key=19, value=10), ds.pair_int_int(key=20, value=11), ds.pair_int_int(key=21, value=11), ds.pair_int_int(key=22, value=12), ds.pair_int_int(key=23, value=12), ds.pair_int_int(key=24, value=13), ds.pair_int_int(key=25, value=13), ds.pair_int_int(key=26, value=14), ds.pair_int_int(key=27, value=14), ds.pair_int_int(key=28, value=15), ds.pair_int_int(key=29, value=15), ds.pair_int_int(key=30, value=16), ds.pair_int_int(key=31, value=16)]))
    func_copy_entry.instances.append(ds.AlimpInstance(width=4, height=1, energy=4, latency=5, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=0), ds.pair_int_int(key=2, value=0), ds.pair_int_int(key=3, value=0), ds.pair_int_int(key=4, value=1), ds.pair_int_int(key=5, value=1), ds.pair_int_int(key=6, value=1), ds.pair_int_int(key=7, value=1), ds.pair_int_int(key=8, value=2), ds.pair_int_int(key=9, value=2), ds.pair_int_int(key=10, value=2), ds.pair_int_int(key=11, value=2), ds.pair_int_int(key=12, value=3), ds.pair_int_int(key=13, value=3), ds.pair_int_int(key=14, value=3), ds.pair_int_int(key=15, value=3)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=1), ds.pair_int_int(key=3, value=1), ds.pair_int_int(key=4, value=2), ds.pair_int_int(key=5, value=2), ds.pair_int_int(key=6, value=2), ds.pair_int_int(key=7, value=2), ds.pair_int_int(key=8, value=3), ds.pair_int_int(key=9, value=3), ds.pair_int_int(key=10, value=3), ds.pair_int_int(key=11, value=3), ds.pair_int_int(key=12, value=4), ds.pair_int_int(key=13, value=4), ds.pair_int_int(key=14, value=4), ds.pair_int_int(key=15, value=4), ds.pair_int_int(key=16, value=5), ds.pair_int_int(key=17, value=5), ds.pair_int_int(key=18, value=5), ds.pair_int_int(key=19, value=5), ds.pair_int_int(key=20, value=6), ds.pair_int_int(key=21, value=6), ds.pair_int_int(key=22, value=6), ds.pair_int_int(key=23, value=6), ds.pair_int_int(key=24, value=7), ds.pair_int_int(key=25, value=7), ds.pair_int_int(key=26, value=7), ds.pair_int_int(key=27, value=7), ds.pair_int_int(key=28, value=8), ds.pair_int_int(key=29, value=8), ds.pair_int_int(key=30, value=8), ds.pair_int_int(key=31, value=8)]))
    db.alimp_lib.entries.append(func_copy_entry)
    func_gx_entry = ds.AlimpEntry()
    func_gx_entry.func = "func_gx"
    func_gx_entry.instances.append(ds.AlimpInstance(width=4, height=4, energy=10, latency=17, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16)]))
    db.alimp_lib.entries.append(func_gx_entry)
    func_gy_entry = ds.AlimpEntry()
    func_gy_entry.func = "func_gy"
    func_gy_entry.instances.append(ds.AlimpInstance(width=4, height=4, energy=10, latency=17, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16)]))
    db.alimp_lib.entries.append(func_gy_entry)
    func_combine_entry = ds.AlimpEntry()
    func_combine_entry.func = "func_combine"
    func_combine_entry.instances.append(ds.AlimpInstance(width=2, height=2, energy=2, latency=17, input_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16), ds.pair_int_int(key=16, value=0), ds.pair_int_int(key=17, value=1), ds.pair_int_int(key=18, value=2), ds.pair_int_int(key=19, value=3), ds.pair_int_int(key=20, value=4), ds.pair_int_int(key=21, value=5), ds.pair_int_int(key=22, value=6), ds.pair_int_int(key=23, value=7), ds.pair_int_int(key=24, value=8), ds.pair_int_int(key=25, value=9), ds.pair_int_int(key=26, value=10), ds.pair_int_int(key=27, value=11), ds.pair_int_int(key=28, value=12), ds.pair_int_int(key=29, value=13), ds.pair_int_int(key=30, value=14), ds.pair_int_int(key=31, value=15)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16)]))
    db.alimp_lib.entries.append(func_combine_entry)
    func_store_entry = ds.AlimpEntry()
    func_store_entry.func = "func_store"
    func_store_entry.instances.append(ds.AlimpInstance(width=4, height=2, energy=1, latency=5, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=0), ds.pair_int_int(key=2, value=0), ds.pair_int_int(key=3, value=0), ds.pair_int_int(key=4, value=1), ds.pair_int_int(key=5, value=1), ds.pair_int_int(key=6, value=1), ds.pair_int_int(key=7, value=1), ds.pair_int_int(key=8, value=2), ds.pair_int_int(key=9, value=2), ds.pair_int_int(key=10, value=2), ds.pair_int_int(key=11, value=2), ds.pair_int_int(key=12, value=3), ds.pair_int_int(key=13, value=3), ds.pair_int_int(key=14, value=3), ds.pair_int_int(key=15, value=3)]))
    db.alimp_lib.entries.append(func_store_entry)


    db.app_graph.nodes.append(ds.AppNode(id="load", func="func_load", output_ports=[ds.AppNodePort(id="load_output", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="copy", func="func_copy", input_ports=[ds.AppNodePort(id="copy_input", rate=1, token_size=16)], output_ports=[ds.AppNodePort(id="copy_output_0", rate=1, token_size=16), ds.AppNodePort(id="copy_output_1", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="gx", func="func_gx", input_ports=[ds.AppNodePort(id="gx_input", rate=1, token_size=16)], output_ports=[ds.AppNodePort(id="gx_output", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="gy", func="func_gy", input_ports=[ds.AppNodePort(id="gy_input", rate=1, token_size=16)], output_ports=[ds.AppNodePort(id="gy_output", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="combine", func="func_combine", input_ports=[ds.AppNodePort(id="combine_input_0", rate=1, token_size=16), ds.AppNodePort(id="combine_input_1", rate=1, token_size=16)], output_ports=[ds.AppNodePort(id="combine_output", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="store", func="func_store", input_ports=[ds.AppNodePort(id="store_input", rate=1, token_size=16)]))

    db.app_graph.edges.append(ds.AppEdge(id="edge_load_copy", source_node="load", target_node="copy", source_port="load_output", target_port="copy_input", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="edge_copy_gx", source_node="copy", target_node="gx", source_port="copy_output_0", target_port="gx_input", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="edge_copy_gy", source_node="copy", target_node="gy", source_port="copy_output_1", target_port="gy_input", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="edge_gx_combine", source_node="gx", target_node="combine", source_port="gx_output", target_port="combine_input_0", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="edge_gy_combine", source_node="gy", target_node="combine", source_port="gy_output", target_port="combine_input_1", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="edge_combine_store", source_node="combine", target_node="store", source_port="combine_output", target_port="store_input", token_size=16))

    return db


def minimum()-> ds.DataBase:
    db = ds.DataBase()
    db.global_constraint.max_energy = 100
    db.global_constraint.max_width = 100
    db.global_constraint.max_height = 100
    db.global_constraint.max_latency = 100
    db.global_constraint.max_period = 70

    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1
    db.hyper_parameter.place_reserved_routing_size = 1

    A_entry = ds.AlimpEntry()
    A_entry.func = "FA"
    A_entry.instances.append(ds.AlimpInstance(width=2, height=2, energy=1, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)]))
    db.alimp_lib.entries.append(A_entry)
    B_entry = ds.AlimpEntry()
    B_entry.func = "FB"
    B_entry.instances.append(ds.AlimpInstance(width=1, height=1, energy=1, latency=17, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)], output_addr_time_patterns=[ds.pair_int_int(key=0, value=1), ds.pair_int_int(key=1, value=2), ds.pair_int_int(key=2, value=3), ds.pair_int_int(key=3, value=4), ds.pair_int_int(key=4, value=5), ds.pair_int_int(key=5, value=6), ds.pair_int_int(key=6, value=7), ds.pair_int_int(key=7, value=8), ds.pair_int_int(key=8, value=9), ds.pair_int_int(key=9, value=10), ds.pair_int_int(key=10, value=11), ds.pair_int_int(key=11, value=12), ds.pair_int_int(key=12, value=13), ds.pair_int_int(key=13, value=14), ds.pair_int_int(key=14, value=15), ds.pair_int_int(key=15, value=16)]))
    db.alimp_lib.entries.append(B_entry)
    C_entry = ds.AlimpEntry()
    C_entry.func = "FC"
    C_entry.instances.append(ds.AlimpInstance(width=4, height=4, energy=10, latency=16, input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7), ds.pair_int_int(key=8, value=8), ds.pair_int_int(key=9, value=9), ds.pair_int_int(key=10, value=10), ds.pair_int_int(key=11, value=11), ds.pair_int_int(key=12, value=12), ds.pair_int_int(key=13, value=13), ds.pair_int_int(key=14, value=14), ds.pair_int_int(key=15, value=15)]))
    db.alimp_lib.entries.append(C_entry)

    db.app_graph.nodes.append(ds.AppNode(id="A", func="FA", output_ports=[ds.AppNodePort(id="A:ab", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="B", func="FB", input_ports=[ds.AppNodePort(id="B:ab", rate=1, token_size=16)], output_ports=[ds.AppNodePort(id="B:bc", rate=1, token_size=16)]))
    db.app_graph.nodes.append(ds.AppNode(id="C", func="FC", input_ports=[ds.AppNodePort(id="C:bc", rate=1, token_size=16)]))

    db.app_graph.edges.append(ds.AppEdge(id="A_B", source_node="A", target_node="B", source_port="A:ab", target_port="B:ab", token_size=16))
    db.app_graph.edges.append(ds.AppEdge(id="B_C", source_node="B", target_node="C", source_port="B:bc", target_port="C:bc", token_size=16))

    return db

def af() -> ds.DataBase:
    db = ds.DataBase()
    
    db.global_constraint.max_energy = 100
    db.global_constraint.max_width = 100
    db.global_constraint.max_height = 100
    db.global_constraint.max_latency = 100
    db.global_constraint.max_period = 70

    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1
    db.hyper_parameter.place_reserved_routing_size = 1

    ecg_input = ds.AlimpEntry()
    age_input = ds.AlimpEntry()
    conv_1 = ds.AlimpEntry()
    bn_conv_1 = ds.AlimpEntry()
    conv_act_1 = ds.AlimpEntry()
    pool_1 = ds.AlimpEntry()
    conv_2 = ds.AlimpEntry()
    bn_conv_2 = ds.AlimpEntry()
    conv_act_2 = ds.AlimpEntry()
    pool_2 = ds.AlimpEntry()
    concatenate_1 = ds.AlimpEntry()
    dense_1 = ds.AlimpEntry()
    bn_dense_1 = ds.AlimpEntry()
    dense_act_1 = ds.AlimpEntry()
    dense_3 = ds.AlimpEntry()
    bn_dense_3 = ds.AlimpEntry()
    dense_act_3 = ds.AlimpEntry()
    output_dense = ds.AlimpEntry()
    output_softmax = ds.AlimpEntry()

    ecg_input.func = "ecg_input_500x2"
    conv_1.func = "conv_500x2_3x8"
    bn_conv_1.func = "batch_norm_498x8"
    conv_act_1.func = "relu_498x8"
    pool_1.func = "max_pool_498x8_3"
    conv_2.func = "conv_166x8_3x4"
    bn_conv_2.func = "batch_norm_164x4"
    conv_act_2.func = "relu_164x4"
    pool_2.func = "max_pool_164x4_3"
    concatenate_1.func = "concatenate_216_1"
    dense_1.func = "dense_217_32"
    bn_dense_1.func = "batch_norm_32"
    dense_act_1.func = "relu_32"
    dense_3.func = "dense_32_16"
    bn_dense_3.func = "batch_norm_16"
    dense_act_3.func = "relu_16"
    output_dense.func = "dense_16_5"
    output_softmax.func = "softmax_5"

    db.alimp_lib.entries.append(ecg_input)
    db.alimp_lib.entries.append(conv_1)
    db.alimp_lib.entries.append(bn_conv_1)
    db.alimp_lib.entries.append(conv_act_1)
    db.alimp_lib.entries.append(pool_1)
    db.alimp_lib.entries.append(conv_2)
    db.alimp_lib.entries.append(bn_conv_2)
    db.alimp_lib.entries.append(conv_act_2)
    db.alimp_lib.entries.append(pool_2)
    db.alimp_lib.entries.append(concatenate_1)
    db.alimp_lib.entries.append(dense_1)
    db.alimp_lib.entries.append(bn_dense_1)
    db.alimp_lib.entries.append(dense_act_1)
    db.alimp_lib.entries.append(dense_3)
    db.alimp_lib.entries.append(bn_dense_3)
    db.alimp_lib.entries.append(dense_act_3)
    db.alimp_lib.entries.append(output_dense)
    db.alimp_lib.entries.append(output_softmax)
    return db

def chf() -> ds.DataBase:
    db = ds.DataBase()
    
    db.global_constraint.max_energy = 1000000
    db.global_constraint.max_width = 1000
    db.global_constraint.max_height = 1000
    db.global_constraint.max_latency = 100000
    db.global_constraint.max_period = 700000

    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1
    db.hyper_parameter.place_reserved_routing_size = 1

    model_input = ds.AlimpEntry()
    dense_1 = ds.AlimpEntry()
    act_1 = ds.AlimpEntry()
    dense_2 = ds.AlimpEntry()
    act_2 = ds.AlimpEntry()
    dense_3 = ds.AlimpEntry()
    act_3 = ds.AlimpEntry()
    dense_4 = ds.AlimpEntry()
    act_4 = ds.AlimpEntry()
    output_dense = ds.AlimpEntry()
    model_output = ds.AlimpEntry()

    model_input.func = "input_24"
    model_input.instances.append(ds.AlimpInstance(width=10, height=30, energy=10, latency=2, output_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)], input_addr_time_patterns=[]))
    dense_1.func = "dense_24_128"
    dense_1.instances.append(ds.AlimpInstance(width=10, height=30, energy=233, latency=3072, output_addr_time_patterns=[ds.pair_int_int(key=0, value=3064), ds.pair_int_int(key=1, value=3065), ds.pair_int_int(key=2, value=3066), ds.pair_int_int(key=3, value=3067), ds.pair_int_int(key=4, value=3068), ds.pair_int_int(key=5, value=3069), ds.pair_int_int(key=6, value=3070), ds.pair_int_int(key=7, value=3071)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    dense_1.instances.append(ds.AlimpInstance(width=20, height=30, energy=233, latency=1536, output_addr_time_patterns=[ds.pair_int_int(key=0, value=1528), ds.pair_int_int(key=1, value=1529), ds.pair_int_int(key=2, value=1530), ds.pair_int_int(key=3, value=1531), ds.pair_int_int(key=4, value=1532), ds.pair_int_int(key=5, value=1533), ds.pair_int_int(key=6, value=1534), ds.pair_int_int(key=7, value=1535)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    dense_1.instances.append(ds.AlimpInstance(width=40, height=30, energy=233, latency=768, output_addr_time_patterns=[ds.pair_int_int(key=0, value=760), ds.pair_int_int(key=1, value=761), ds.pair_int_int(key=2, value=762), ds.pair_int_int(key=3, value=763), ds.pair_int_int(key=4, value=764), ds.pair_int_int(key=5, value=765), ds.pair_int_int(key=6, value=766), ds.pair_int_int(key=7, value=767)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    
    act_1.func = "act_128"
    act_1.instances.append(ds.AlimpInstance(width=10, height=30, energy=10, latency=128, output_addr_time_patterns=[ds.pair_int_int(key=0, value=120), ds.pair_int_int(key=1, value=121), ds.pair_int_int(key=2, value=122), ds.pair_int_int(key=3, value=123), ds.pair_int_int(key=4, value=124), ds.pair_int_int(key=5, value=125), ds.pair_int_int(key=6, value=126), ds.pair_int_int(key=7, value=127)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    act_1.instances.append(ds.AlimpInstance(width=20, height=30, energy=10, latency=64, output_addr_time_patterns=[ds.pair_int_int(key=0, value=60), ds.pair_int_int(key=1, value=60), ds.pair_int_int(key=2, value=61), ds.pair_int_int(key=3, value=61), ds.pair_int_int(key=4, value=62), ds.pair_int_int(key=5, value=62), ds.pair_int_int(key=6, value=63), ds.pair_int_int(key=7, value=63)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    act_1.instances.append(ds.AlimpInstance(width=40, height=30, energy=10, latency=23, output_addr_time_patterns=[ds.pair_int_int(key=0, value=30), ds.pair_int_int(key=1, value=30), ds.pair_int_int(key=2, value=30), ds.pair_int_int(key=3, value=30), ds.pair_int_int(key=4, value=31), ds.pair_int_int(key=5, value=31), ds.pair_int_int(key=6, value=31), ds.pair_int_int(key=7, value=31)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    
    dense_2.func = "dense_128_64"
    dense_2.instances.append(ds.AlimpInstance(width=10, height=30, energy=622, latency=8192, output_addr_time_patterns=[ds.pair_int_int(key=0, value=8188), ds.pair_int_int(key=1, value=8189), ds.pair_int_int(key=2, value=8190), ds.pair_int_int(key=3, value=8191)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    dense_2.instances.append(ds.AlimpInstance(width=20, height=30, energy=622, latency=4096, output_addr_time_patterns=[ds.pair_int_int(key=0, value=4094), ds.pair_int_int(key=1, value=4094), ds.pair_int_int(key=2, value=4095), ds.pair_int_int(key=3, value=4095)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    dense_2.instances.append(ds.AlimpInstance(width=40, height=30, energy=622, latency=2048, output_addr_time_patterns=[ds.pair_int_int(key=0, value=2047), ds.pair_int_int(key=1, value=2047), ds.pair_int_int(key=2, value=2047), ds.pair_int_int(key=3, value=2047)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4), ds.pair_int_int(key=5, value=5), ds.pair_int_int(key=6, value=6), ds.pair_int_int(key=7, value=7)]))
    


    act_2.func = "act_64"
    act_2.instances.append(ds.AlimpInstance(width=10, height=30, energy=4, latency=64, output_addr_time_patterns=[ds.pair_int_int(key=0, value=60), ds.pair_int_int(key=1, value=61), ds.pair_int_int(key=2, value=62), ds.pair_int_int(key=3, value=63)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3)]))
    act_2.instances.append(ds.AlimpInstance(width=20, height=30, energy=4, latency=32, output_addr_time_patterns=[ds.pair_int_int(key=0, value=30), ds.pair_int_int(key=1, value=30), ds.pair_int_int(key=2, value=31), ds.pair_int_int(key=3, value=31)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3)]))
    act_2.instances.append(ds.AlimpInstance(width=40, height=30, energy=4, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=15), ds.pair_int_int(key=1, value=15), ds.pair_int_int(key=2, value=15), ds.pair_int_int(key=3, value=15)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3)]))
    
    dense_3.func = "dense_64_32"
    dense_3.instances.append(ds.AlimpInstance(width=10, height=30, energy=155, latency=2048, output_addr_time_patterns=[ds.pair_int_int(key=0, value=2046), ds.pair_int_int(key=1, value=2047)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4)]))
    dense_3.instances.append(ds.AlimpInstance(width=20, height=30, energy=155, latency=1024, output_addr_time_patterns=[ds.pair_int_int(key=0, value=1023), ds.pair_int_int(key=1, value=2043)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4)]))
    dense_3.instances.append(ds.AlimpInstance(width=40, height=30, energy=155, latency=512, output_addr_time_patterns=[ds.pair_int_int(key=0, value=512), ds.pair_int_int(key=1, value=512)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1), ds.pair_int_int(key=2, value=2), ds.pair_int_int(key=3, value=3), ds.pair_int_int(key=4, value=4)]))
    
    act_3.func = "act_32"
    act_3.instances.append(ds.AlimpInstance(width=10, height=30, energy=1, latency=32, output_addr_time_patterns=[ds.pair_int_int(key=0, value=30),ds.pair_int_int(key=1, value=31)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0),ds.pair_int_int(key=1, value=0)]))
    act_3.instances.append(ds.AlimpInstance(width=20, height=30, energy=1, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=15),ds.pair_int_int(key=1, value=15)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0),ds.pair_int_int(key=1, value=0)]))
    act_3.instances.append(ds.AlimpInstance(width=40, height=30, energy=1, latency=8, output_addr_time_patterns=[ds.pair_int_int(key=0, value=7),ds.pair_int_int(key=1, value=7)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0),ds.pair_int_int(key=1, value=0)]))
    
    dense_4.func = "dense_32_16"
    dense_4.instances.append(ds.AlimpInstance(width=10, height=30, energy=31, latency=512, output_addr_time_patterns=[ds.pair_int_int(key=0, value=511)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    dense_4.instances.append(ds.AlimpInstance(width=20, height=30, energy=31, latency=256, output_addr_time_patterns=[ds.pair_int_int(key=0, value=255)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    dense_4.instances.append(ds.AlimpInstance(width=40, height=30, energy=31, latency=128, output_addr_time_patterns=[ds.pair_int_int(key=0, value=127)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0), ds.pair_int_int(key=1, value=1)]))
    

    act_4.func = "act_16"
    act_4.instances.append(ds.AlimpInstance(width=10, height=30, energy=1, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=15)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    act_4.instances.append(ds.AlimpInstance(width=20, height=30, energy=1, latency=8, output_addr_time_patterns=[ds.pair_int_int(key=0, value=7)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    act_4.instances.append(ds.AlimpInstance(width=40, height=30, energy=1, latency=4, output_addr_time_patterns=[ds.pair_int_int(key=0, value=3)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    
    output_dense.func = "dense_16_2"
    output_dense.instances.append(ds.AlimpInstance(width=10, height=30, energy=2, latency=32, output_addr_time_patterns=[ds.pair_int_int(key=0, value=31)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    output_dense.instances.append(ds.AlimpInstance(width=20, height=30, energy=2, latency=16, output_addr_time_patterns=[ds.pair_int_int(key=0, value=15)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    output_dense.instances.append(ds.AlimpInstance(width=40, height=30, energy=2, latency=8, output_addr_time_patterns=[ds.pair_int_int(key=0, value=7)], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))
    
    model_output.func = "output_2"
    model_output.instances.append(ds.AlimpInstance(width=10, height=30, energy=1, latency=1, output_addr_time_patterns=[], input_addr_time_patterns=[ds.pair_int_int(key=0, value=0)]))

    db.alimp_lib.entries.append(model_input)
    db.alimp_lib.entries.append(dense_1)
    db.alimp_lib.entries.append(act_1)
    db.alimp_lib.entries.append(dense_2)
    db.alimp_lib.entries.append(act_2)
    db.alimp_lib.entries.append(dense_3)
    db.alimp_lib.entries.append(act_3)
    db.alimp_lib.entries.append(dense_4)
    db.alimp_lib.entries.append(act_4)
    db.alimp_lib.entries.append(output_dense)
    db.alimp_lib.entries.append(model_output)

    db.app_graph.nodes.append(ds.AppNode(id="model_input", func="input_24", output_ports=[ds.AppNodePort(id="model_input_out", rate=1, token_size=2)]))
    db.app_graph.nodes.append(ds.AppNode(id="dense_1", func="dense_24_128", input_ports=[ds.AppNodePort(id="dense_1_in", rate=1, token_size=2)], output_ports=[ds.AppNodePort(id="dense_1_out", rate=1, token_size=8)]))
    db.app_graph.nodes.append(ds.AppNode(id="act_1", func="act_128", input_ports=[ds.AppNodePort(id="act_1_in", rate=1, token_size=8)], output_ports=[ds.AppNodePort(id="act_1_out", rate=1, token_size=8)]))
    db.app_graph.nodes.append(ds.AppNode(id="dense_2", func="dense_128_64", input_ports=[ds.AppNodePort(id="dense_2_in", rate=1, token_size=8)], output_ports=[ds.AppNodePort(id="dense_2_out", rate=1, token_size=4)]))
    db.app_graph.nodes.append(ds.AppNode(id="act_2", func="act_64", input_ports=[ds.AppNodePort(id="act_2_in", rate=1, token_size=4)], output_ports=[ds.AppNodePort(id="act_2_out", rate=1, token_size=4)]))
    db.app_graph.nodes.append(ds.AppNode(id="dense_3", func="dense_64_32", input_ports=[ds.AppNodePort(id="dense_3_in", rate=1, token_size=4)], output_ports=[ds.AppNodePort(id="dense_3_out", rate=1, token_size=2)]))
    db.app_graph.nodes.append(ds.AppNode(id="act_3", func="act_32", input_ports=[ds.AppNodePort(id="act_3_in", rate=1, token_size=2)], output_ports=[ds.AppNodePort(id="act_3_out", rate=1, token_size=2)]))
    db.app_graph.nodes.append(ds.AppNode(id="dense_4", func="dense_32_16", input_ports=[ds.AppNodePort(id="dense_4_in", rate=1, token_size=2)], output_ports=[ds.AppNodePort(id="dense_4_out", rate=1, token_size=1)]))
    db.app_graph.nodes.append(ds.AppNode(id="act_4", func="act_16", input_ports=[ds.AppNodePort(id="act_4_in", rate=1, token_size=1)], output_ports=[ds.AppNodePort(id="act_4_out", rate=1, token_size=1)]))
    db.app_graph.nodes.append(ds.AppNode(id="output_dense", func="dense_16_2", input_ports=[ds.AppNodePort(id="output_dense_in", rate=1, token_size=1)], output_ports=[ds.AppNodePort(id="output_dense_out", rate=1, token_size=1)]))
    db.app_graph.nodes.append(ds.AppNode(id="model_output", func="output_2", input_ports=[ds.AppNodePort(id="model_output_in", rate=1, token_size=1)]))

    db.app_graph.edges.append(ds.AppEdge(id="edge_model_input_dense_1", source_node="model_input", target_node="dense_1", source_port="model_input_out", target_port="dense_1_in", token_size=2))
    db.app_graph.edges.append(ds.AppEdge(id="edge_dense_1_act_1", source_node="dense_1", target_node="act_1", source_port="dense_1_out", target_port="act_1_in", token_size=8))
    db.app_graph.edges.append(ds.AppEdge(id="edge_act_1_dense_2", source_node="act_1", target_node="dense_2", source_port="act_1_out", target_port="dense_2_in", token_size=8))
    db.app_graph.edges.append(ds.AppEdge(id="edge_dense_2_act_2", source_node="dense_2", target_node="act_2", source_port="dense_2_out", target_port="act_2_in", token_size=4))
    db.app_graph.edges.append(ds.AppEdge(id="edge_act_2_dense_3", source_node="act_2", target_node="dense_3", source_port="act_2_out", target_port="dense_3_in", token_size=4))
    db.app_graph.edges.append(ds.AppEdge(id="edge_dense_3_act_3", source_node="dense_3", target_node="act_3", source_port="dense_3_out", target_port="act_3_in", token_size=2))
    db.app_graph.edges.append(ds.AppEdge(id="edge_act_3_dense_4", source_node="act_3", target_node="dense_4", source_port="act_3_out", target_port="dense_4_in", token_size=2))
    db.app_graph.edges.append(ds.AppEdge(id="edge_dense_4_act_4", source_node="dense_4", target_node="act_4", source_port="dense_4_out", target_port="act_4_in", token_size=1))
    db.app_graph.edges.append(ds.AppEdge(id="edge_act_4_output_dense", source_node="act_4", target_node="output_dense", source_port="act_4_out", target_port="output_dense_in", token_size=1))
    db.app_graph.edges.append(ds.AppEdge(id="edge_output_dense_model_output", source_node="output_dense", target_node="model_output", source_port="output_dense_out", target_port="model_output_in", token_size=1))

    return db

def random_test() -> ds.DataBase:
    db = ds.DataBase()

    db.global_constraint.max_energy = 1000
    db.global_constraint.max_width = 1000
    db.global_constraint.max_height = 1000
    db.global_constraint.max_latency = 1000
    db.global_constraint.max_period = 1000

    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1
    db.hyper_parameter.place_reserved_routing_size = 1

    # generate a random SDF graph
    num_nodes = 5
    edge_node_ratio = 1.0
    num_edges = int(num_nodes * edge_node_ratio)
    max_token_per_edge = 10
    max_token_size = 10
    num_instance_per_func = 4
    max_func_latency = 10
    max_func_energy = 10
    max_func_width = 10
    max_func_height = 10

    output_ports = {}
    input_ports = {}
    port_token_size = {}

    for i in range(num_edges):
        source_id = random.randint(0, num_nodes - 1)
        target_id = random.randint(0, num_nodes - 1)
        while source_id == target_id:
            target_id = random.randint(0, num_nodes - 1)
        if source_id > target_id:
            source_id, target_id = target_id, source_id
        source_node = f"node_{source_id}"
        target_node = f"node_{target_id}"

        if output_ports.get(source_node) is None:
            output_ports[source_node] = []
        if input_ports.get(target_node) is None:
            input_ports[target_node] = []
        source_port = f"{source_node}_out_{len(output_ports[source_node])}"
        target_port = f"{target_node}_in_{len(input_ports[target_node])}"
        output_ports[source_node].append(source_port)
        input_ports[target_node].append(target_port)
        token_size = random.randint(1, max_token_size)
        port_token_size[source_port] = token_size
        port_token_size[target_port] = token_size
        db.app_graph.edges.append(ds.AppEdge(id=f"edge_{source_node}_{target_node}", source_node=source_node, target_node=target_node, source_port=source_port, target_port=target_port, token_size=token_size))

    print(output_ports)
    for i in range(num_nodes):
        node = f"node_{i}"
        func = f"func_{i}"
        node_output_ports = output_ports.get(node)
        if node_output_ports is None:
            node_output_ports = []
        node_input_ports = input_ports.get(node)
        if node_input_ports is None:
            node_input_ports = []
        db.app_graph.nodes.append(ds.AppNode(id=node, func=func, input_ports=[ds.AppNodePort(id=port, rate=1, token_size=port_token_size[port]) for port in node_input_ports], output_ports=[ds.AppNodePort(id=port, rate=1, token_size=port_token_size[port]) for port in node_output_ports]))

        total_out_token_size = 0
        total_in_token_size = 0
        for port in node_output_ports:
            total_out_token_size += port_token_size[port]
        for port in node_input_ports:
            total_in_token_size += port_token_size[port]
        
        entry = ds.AlimpEntry(func=func)
        for i in range(num_instance_per_func):
        # generate a random timeline for token chunks
            input_addr_time_patterns = [random.randint(0, max_func_latency) for _ in range(total_in_token_size)]
            output_addr_time_patterns = [random.randint(0, max_func_latency) for _ in range(total_out_token_size)]
            print(input_addr_time_patterns)
            print(output_addr_time_patterns)
            input_latency = 1
            if len(input_addr_time_patterns) > 0:
                input_latency = max(input_addr_time_patterns) + 1
            output_latency = 1
            if len(output_addr_time_patterns) > 0:
                output_latency = max(output_addr_time_patterns) + 1
            latency = max(input_latency, output_latency)
            energy = random.randint(1, max_func_energy)
            width = random.randint(1, max_func_width)
            if width < total_in_token_size:
                width = total_in_token_size
            if width < total_out_token_size:
                width = total_out_token_size
            height = random.randint(1, max_func_height)
            entry.instances.append(ds.AlimpInstance(width=width, height=height, energy=energy, latency=latency, input_addr_time_patterns=[ds.pair_int_int(key=i, value=input_addr_time_patterns[i]) for i in range(total_in_token_size)], output_addr_time_patterns=[ds.pair_int_int(key=i, value=output_addr_time_patterns[i]) for i in range(total_out_token_size)]))
        db.alimp_lib.entries.append(entry)
    

def read_addr_pattern(filename) -> list :
        patterns = []
        try:
            with open(filename, 'r') as f:
                json_data = json.load(f)
            for pt in json_data['addr_ptrn']:
                patterns.append(ds.pair_int_int(key=int(pt['address']), value=int(pt['cycle'])))
        except:
            print("No address pattern file found, ignore: ", filename)
        return patterns

def lenet5() -> ds.DataBase:
    db = ds.DataBase()
    
    db.global_constraint.max_energy = 10000
    db.global_constraint.max_width = 10000
    db.global_constraint.max_height = 10000
    db.global_constraint.max_latency = 100000
    db.global_constraint.max_period = 100000
    db.hyper_parameter.bind_w_area = 1
    db.hyper_parameter.bind_w_energy = 1
    db.hyper_parameter.bind_w_latency = 1
    db.hyper_parameter.bind_relaxation_factor = 1.1
    db.hyper_parameter.place_relaxation_factor = 1.5
    db.hyper_parameter.place_reserved_routing_size = 1

    prefix = "work/sim/"

    def add_entry_instance(db, name, func, prefix, width, height, energy) -> list:
        entry = ds.AlimpEntry()
        entry.func = func
        input_addr_time_patterns = read_addr_pattern(prefix+name+"_inAP.json")
        output_addr_time_patterns = read_addr_pattern(prefix+name+"_outAP.json")
        input_addr_time_patterns_max = max([pt.value for pt in input_addr_time_patterns]) if len(input_addr_time_patterns) > 0 else 0
        output_addr_time_patterns_max = max([pt.value for pt in output_addr_time_patterns]) if len(output_addr_time_patterns) > 0 else 0
        latency = max(input_addr_time_patterns_max, output_addr_time_patterns_max) + 1
        input_token = len(input_addr_time_patterns)
        output_token = len(output_addr_time_patterns)
        entry.instances.append(ds.AlimpInstance(width=width, height=height, energy=energy, latency=latency, input_addr_time_patterns=input_addr_time_patterns, output_addr_time_patterns=output_addr_time_patterns))
        db.alimp_lib.entries.append(entry)
        print(f"Add entry {name} with {input_token} input tokens and {output_token} output tokens")
        return (input_token, output_token)

    (conv1_input_token, conv1_output_token) = add_entry_instance(db, "conv1", "conv_32x32_5x5", prefix, 2, 2, 10)
    (pooling1_input_token, pooling1_output_token) = add_entry_instance(db, "pooling1", "max_pool_28x28_2", prefix, 10, 10, 10)
    (conv2_input_token, conv2_output_token) = add_entry_instance(db, "conv2", "conv_14x14_5x5", prefix, 10, 10, 10)
    (pooling2_input_token, pooling2_output_token) = add_entry_instance(db, "pooling2", "max_pool_10x10_2", prefix, 10, 10, 10)
    (conv3_input_token, conv3_output_token) = add_entry_instance(db, "conv3", "conv_5x5_5x5", prefix, 5, 5, 10)
    (reshape_input_token, reshape_output_token) = add_entry_instance(db, "reshape", "reshape_5x5_1", prefix, 5, 5, 10)
    (fc1_input_token, fc1_output_token) = add_entry_instance(db, "fc1", "dense_120_84", prefix, 5, 1, 10)
    (fc2_input_token, fc2_output_token) = add_entry_instance(db, "fc2", "dense_84_10", prefix, 5, 1, 10)
    (store_output_input_token, store_output_output_token) = add_entry_instance(db, "store_output", "output_10", prefix, 5, 1, 10)
    (load_input_input_token, load_input_output_token) = add_entry_instance(db, "load_input", "input_32x32", prefix, 5, 5, 10)
    
    db.app_graph.nodes.append(ds.AppNode(id="load_input", func="input_32x32", output_ports=[ds.AppNodePort(id="load_input_out", rate=1, token_size=load_input_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="conv1", func="conv_32x32_5x5", input_ports=[ds.AppNodePort(id="conv1_in", rate=1, token_size=conv1_input_token)], output_ports=[ds.AppNodePort(id="conv1_out", rate=1, token_size=conv1_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="pooling1", func="max_pool_28x28_2", input_ports=[ds.AppNodePort(id="pooling1_in", rate=1, token_size=pooling1_input_token)], output_ports=[ds.AppNodePort(id="pooling1_out", rate=1, token_size=pooling1_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="conv2", func="conv_14x14_5x5", input_ports=[ds.AppNodePort(id="conv2_in", rate=1, token_size=conv2_input_token)], output_ports=[ds.AppNodePort(id="conv2_out", rate=1, token_size=conv2_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="pooling2", func="max_pool_10x10_2", input_ports=[ds.AppNodePort(id="pooling2_in", rate=1, token_size=pooling2_input_token)], output_ports=[ds.AppNodePort(id="pooling2_out", rate=1, token_size=pooling2_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="conv3", func="conv_5x5_5x5", input_ports=[ds.AppNodePort(id="conv3_in", rate=1, token_size=conv3_input_token)], output_ports=[ds.AppNodePort(id="conv3_out", rate=1, token_size=conv3_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="reshape", func="reshape_5x5_1", input_ports=[ds.AppNodePort(id="reshape_in", rate=1, token_size=reshape_input_token)], output_ports=[ds.AppNodePort(id="reshape_out", rate=1, token_size=reshape_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="fc1", func="dense_120_84", input_ports=[ds.AppNodePort(id="fc1_in", rate=1, token_size=fc1_input_token)], output_ports=[ds.AppNodePort(id="fc1_out", rate=1, token_size=fc1_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="fc2", func="dense_84_10", input_ports=[ds.AppNodePort(id="fc2_in", rate=1, token_size=fc2_input_token)], output_ports=[ds.AppNodePort(id="fc2_out", rate=1, token_size=fc2_output_token)]))
    db.app_graph.nodes.append(ds.AppNode(id="store_output", func="output_10", input_ports=[ds.AppNodePort(id="store_output_in", rate=1, token_size=store_output_output_token)]))
    db.app_graph.edges.append(ds.AppEdge(id="edge_load_input_conv1", source_node="load_input", target_node="conv1", source_port="load_input_out", target_port="conv1_in", token_size=load_input_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_conv1_pooling1", source_node="conv1", target_node="pooling1", source_port="conv1_out", target_port="pooling1_in", token_size=conv1_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_pooling1_conv2", source_node="pooling1", target_node="conv2", source_port="pooling1_out", target_port="conv2_in", token_size=pooling1_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_conv2_pooling2", source_node="conv2", target_node="pooling2", source_port="conv2_out", target_port="pooling2_in", token_size=conv2_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_pooling2_conv3", source_node="pooling2", target_node="conv3", source_port="pooling2_out", target_port="conv3_in", token_size=pooling2_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_conv3_reshape", source_node="conv3", target_node="reshape", source_port="conv3_out", target_port="reshape_in", token_size=conv3_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_reshape_fc1", source_node="reshape", target_node="fc1", source_port="reshape_out", target_port="fc1_in", token_size=reshape_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_fc1_fc2", source_node="fc1", target_node="fc2", source_port="fc1_out", target_port="fc2_in", token_size=fc1_output_token))
    db.app_graph.edges.append(ds.AppEdge(id="edge_fc2_store_output", source_node="fc2", target_node="store_output", source_port="fc2_out", target_port="store_output_in", token_size=fc2_output_token))


    return db
