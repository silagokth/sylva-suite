from google.protobuf.json_format import MessageToJson
import sys
import os

from ds import AlgorithmDataFlowGraph as ADFG_api
from ds import AlgorithmDataFlowGraph_pb2 as ADFG_pb2

b = ADFG_api.AlgorithmDataFlowGraph_Handler()
a = ADFG_pb2.AlgorithmDataFlowGraph()

b.save_json(a, "a.json")
