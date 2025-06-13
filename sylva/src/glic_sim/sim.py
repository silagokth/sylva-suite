#! /usr/bin/python3

## @package main
#  Main file to run the GLIC simulation. Needs glic_config file.

import os
import sys
import copy
import json
import argparse

from lib.glic_sim.base_module import base_module
from lib.glic_sim.common import *
from src.glic_sim.node import node

import lib.glic_sim.proto.glic_config_pb2 as gConfig
from google.protobuf.json_format import MessageToJson, Parse, ParseDict

_VERBOSITY = verbosity.NONE
_CONFIG = ''
_TIMETABLE = ''
_WORKSPACE_PATH = ''
_USE_JSON = False

class top(base_module):

  def __init__(self, v, config, timetable, workspace, json):
    super(top, self).__init__("Top", moduleType.TopModule, v)
    self.m_node_insts = {}
    self.nMap = gConfig.node_config_map()
    self.nTT  = gConfig.time_table()
    self.config = config
    self.timetable = timetable
    self.workspace = workspace
    self.json = json

  def set_verbosity(self, ver):
    base_module.set_verbosity(self, ver)
    for n in self.m_node_insts:
      self.m_node_insts[n].set_verbosity(ver)

  def initialise(self):
    try:
      # @TODO: Make this work with nojson flag
      with open(self.config, 'r') as configFile:
        # print(json.load(configFile))
        ParseDict(json.load(configFile), self.nMap)
        # Parse(configFile.read(), self.nMap)
    except Exception as e:
      print("\n\nCONFIG FILE NOT FOUND!\n(Path=%s)\n\n\n" % self.config)
      raise e

    self.infoDEBUG("Fetched Config:", self.nMap)

    try:
      with open(self.timetable) as tt:
        ParseDict(json.load(tt), self.nTT)
    except Exception as e:
      print("\n\nTIME TABLE FILE NOT FOUND!\n(Path=%s)\n\n\n" % self.timetable)
      raise e

    for name in self.nMap.config_map:
      self.infoDEBUG(f"Instantiating: {name}.")
      self.m_node_insts[name] = node(name
                                    , self.nMap.config_map[name]
                                    , self.workspace
                                    , self.json
                                    , self.m_my_verbosity)

    # Create mem folder to store memory images and buffers.
    self.cmd_path = self.workspace
    self.cstm_env = os.environ.copy()
    self.execute_command("mkdir -p mem", _shell=False)
    self.infoMEDIUM(f"Initiated {len(self.m_node_insts)} nodes.")
    self.infoDEBUG(f"command path = {self.cmd_path}")

  def doYourThing(self, globalTime):
    doneList = []
    todoList = list(self.nMap.config_map.keys())
    self.infoMEDIUM(f"todoList Formulated.")
    self.infoHIGH(f"{todoList}")

    iterNr  = 0
    maxIter = len(todoList)
    while todoList:
      found = False
      assert(++iterNr < maxIter)
      self.infoDEBUG(f"todo: {todoList}\ndone:{doneList}")
      for nodeName in todoList:
          self.infoDEBUG(f"Trying: {nodeName}. inP:  {self.nMap.config_map[nodeName].in_names}, isTr: {self.nMap.config_map[nodeName].is_transporter}")
        #if not self.nMap.config_map[nodeName].is_transporter:
        #  self.infoHIGH(f"Trying: {nodeName}. inP: {self.nMap.config_map[nodeName].in_names}")
          if ((not len(self.nMap.config_map[nodeName].in_names))
              or (all(nNm in doneList for nNm in self.nMap.config_map[nodeName].in_names))):
            found = True
            # TODO: execute all fire cycles at once
            ttInst = next((inst for inst in self.nTT.tt if inst.node_name == nodeName), None)
            assert(ttInst != None)
            globalTime = ttInst.cycle
            self.infoMEDIUM(f"ttInst: {ttInst}.")
            self.infoLOW(f"@[{globalTime}]Triggering: {nodeName}.")
            self.m_node_insts[nodeName].doYourThing(globalTime)
            doneList.append(nodeName)
            todoList.remove(nodeName)
            self.infoLOW(f"Finishing: {nodeName}.")
            ''' Not separating transporters from normal nodes
            for outs in self.nMap.config_map[nodeName].out_names:
              assert(self.nMap.config_map[outs].is_transporter)
              ttInst = next((inst for inst in self.nTT.tt if inst.node_name == outs), None)
              assert(ttInst != None)
              self.infoMEDIUM(f"ttInst: {ttInst}.")
              self.infoLOW(f"@[{globalTime}]Triggering: {outs}.")
              self.m_node_insts[outs].doYourThing(ttInst.cycle)
              globalTime += ttInst.cycle
              doneList.append(outs)
              todoList.remove(outs)
              self.infoLOW(f"@[{globalTime}]Done: {outs}.")
            '''
      assert(found == True)

if __name__ == "__main__":
  ''' Add all the switches available with this script to help
  '' and fetch them in args variable.
  '''
  parser=argparse.ArgumentParser(
      description='',
      epilog="")

  parser.add_argument('--verbosity'
                    , default="NONE"
                    , choices=["NONE", "LOW", "MEDIUM", "HIGH", "FULL", "DEBUG"]
                    , help='Make The script verbose'
                    )

  parser.add_argument('--nojson'
                    , action="store_true"
                    , default=False
                    , help='Use Binary (instead of json) files for protobuf operations.'
                    )

  parser.add_argument('--configJSON'
                    , type=str
                    , required=True
                    , help='Full path of a JSON config file, used to fetch node_config_map.'
                    )

  parser.add_argument('--timeTableJSON'
                    , type=str
                    , required=True
                    , help='Full path of a JSON time table file, used to fire nodes.'
                    )

  parser.add_argument('--workspacePath'
                    , type=str
                    , default="./"
                    , help='Path to the folder where all memory & instruction files are to be located.'
                    )

  args=parser.parse_args()

  if args.verbosity == "LOW":
    _VERBOSITY = verbosity.LOW
  elif args.verbosity == "MEDIUM":
    _VERBOSITY = verbosity.MEDIUM
  elif args.verbosity == "HIGH":
    _VERBOSITY = verbosity.HIGH
  elif args.verbosity == "FULL":
    _VERBOSITY = verbosity.FULL
  elif args.verbosity == "DEBUG":
    _VERBOSITY = verbosity.DEBUG
  else:
    _VERBOSITY = verbosity.NONE

  _CONFIG = args.configJSON
  _TIMETABLE = args.timeTableJSON
  _WORKSPACE_PATH =  args.workspacePath

  _USE_JSON = not args.nojson

  globalCycle = 0
  print("-----------------------------------")
  
  top_inst = top(_VERBOSITY, _CONFIG, _TIMETABLE, _WORKSPACE_PATH, _USE_JSON)
  top_inst.initialise()
  top_inst.doYourThing(globalCycle)
