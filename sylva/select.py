# select Alimps

import os
import sys
import json

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import Parse

import data_structure_pb2 as ds


def create_alimp_library(db):
    db.alimp_library = []

    alimp_instance = ds.AlimpInstance(id="conv_3x3_1", )
    alimp_instance.id = "conv_3x3_1"


def select(db, num) -> []:
    ''' Select alimp for each computation nodes in app_graph. It returns a list of valid db of valid assignment. We use genetic algorithm to find the best assignments.
    '''
