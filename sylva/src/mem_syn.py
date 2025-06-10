#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import lib.proto.data_structure_pb2 as ds
import copy
import matplotlib.pyplot as plt
import random
import json
import re

from google.protobuf.json_format import MessageToJson
from google.protobuf.json_format import MessageToDict
from google.protobuf.json_format import Parse

import logging

def run(db: ds.DataBase, output_dir: str) -> bool:
    logging.info("Start: memory synthesis")
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    sim_dir = os.path.join(output_dir, 'mem_syn')
    logging.info("Finish: memory synthesis")
    return True
