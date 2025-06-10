#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from ds import AlgorithmDataFlowGraph_pb2
from google.protobuf.json_format import MessageToJson


class AlgorithmDataFlowGraph_Handler:
    def __init__(self):
        pass

    def load(self, filename: str) -> AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph:
        if not os.path.exists(filename):
            print("Error: file not exists: %s" % filename)
            return None
        g = AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph()
        with open(filename, "rb") as f:
            g.ParseFromString(f.read())
        return g

    def save(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph, filename: str) -> None:
        with open(filename, "wb") as f:
            f.write(g.SerializeToString())
        return True

    def load_json(self, filename: str) -> AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph:
        if not os.path.exists(filename):
            print("Error: file not exists: %s" % filename)
            return None
        g = AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph()
        with open(filename, "r") as f:
            g.ParseFromString(f.read())
        return g

    def save_json(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph, filename: str) -> None:
        with open(filename, "w") as f:
            f.write(MessageToJson(g))

    def num_vertices(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph):
        return len(g.vertices)

    def num_edges(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph):
        return len(g.edges)

    def get_vertex(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph, id: int):
        return g.vertices[id]

    def get_edge(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph, id: int):
        return g.edges[id]

    def get_vertices_by_func(self, g: AlgorithmDataFlowGraph_pb2.AlgorithmDataFlowGraph, func: str):
        vertices = []
        for v in g.vertices:
            if v.func == func:
                vertices.append(v)
        return vertices
