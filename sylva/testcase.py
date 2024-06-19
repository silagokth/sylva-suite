import sys
import os
import data_structure_pb2 as ds
from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

def create_test_db(testcase_name:str):
    this_module = sys.modules[__name__]
    func = getattr(this_module, testcase_name)

    db = func()
    # write db.app_graph to json file
    app_graph_json = MessageToJson(db.app_graph)
    with open(os.path.join('app_graph.json'), 'w') as f:
        f.write(app_graph_json)
    # write db.global_constraint to json file
    global_constraint_json = MessageToJson(db.global_constraint)
    with open(os.path.join('global_constraint.json'), 'w') as f:
        f.write(global_constraint_json)
    # write db.alimp_lib to json file
    alimp_lib_json = MessageToJson(db.alimp_lib)
    with open(os.path.join('alimp_lib.json'), 'w') as f:
        f.write(alimp_lib_json)
    # write db.hyper_parameter to json file
    hyper_parameter_json = MessageToJson(db.hyper_parameter)
    with open(os.path.join('hyper_parameter.json'), 'w') as f:
        f.write(hyper_parameter_json)

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
