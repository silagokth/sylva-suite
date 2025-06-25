## @package node
#  This is a node. It contains process & addrTranslators.
# Depending on the config it can have either both input
# and output addrTranslators or one or the other.
import json
import shlex 

from lib.glic_sim.common import *
from lib.glic_sim.base_module import base_module
from src.glic_sim.input_addr_translator import input_addr_translator
from src.glic_sim.process_module import process_module
from src.glic_sim.output_addr_translator import output_addr_translator
from src.glic_sim.transporter_module import transporter_module

import lib.glic_sim.proto.glic_config_pb2 as gConfig
import lib.glic_sim.proto.buffer_pb2 as memBfr
from google.protobuf.json_format import MessageToJson, Parse, ParseDict

class node(base_module):
  def set_verbosity(self, ver):
    base_module.set_verbosity(self, ver)
    if self.m_in_addrT != None:
      self.m_in_addrT.set_verbosity(ver)
    if self.m_process != None:
      self.m_process.set_verbosity(ver)
    if self.m_out_addrT != None:
      self.m_out_addrT.set_verbosity(ver)
    if self.m_transporter != None:
      self.m_transporter.set_verbosity(ver)

  def __init__(self, name, cfg, path, useJson, v):
    super(node, self).__init__(name, moduleType.node, v)
    self.m_config      = gConfig.node_config()
    self.m_use_json    = useJson
    self.m_in_addrT    = None
    self.m_process     = None
    self.m_out_addrT   = None
    self.m_transporter = None
    self.m_path        = path
    self.m_config.CopyFrom(cfg)
    self.infoDEBUG(f"my config = {self.m_config}")

    if not self.m_config.is_transporter:
      # add relative path to the process command
      self.m_config.process_cmd = self.m_config.process_cmd
      self.m_config.process_cmd += f" --global-image ./{self.m_path}/mem/global_mem_image.json"
      #self.m_config.process_cmd += f" ./{self.m_path}/mem/global_mem_image.json"
      if len(self.m_config.in_names) != 0:
        self.m_in_addrT = input_addr_translator(self.m_my_name, path, useJson, v)
        self.m_config.process_cmd += f" --in-mem ./{self.m_path}/mem/{self.m_my_name}_inMem.json"
        #self.m_config.process_cmd += f" ./{self.m_path}/mem/{self.m_my_name}_inMem.json"
      if len(self.m_config.out_names) != 0:
        self.m_out_addrT = output_addr_translator(self.m_my_name, path, useJson, v)
        self.m_config.process_cmd += f" --out-mem ./{self.m_path}/mem/{self.m_my_name}_outMem.json"
        #self.m_config.process_cmd += f" ./{self.m_path}/mem/{self.m_my_name}_outMem.json"
      # let the process run the executables from the main dir
      self.m_process = process_module(self.m_my_name, "", self.m_config.process_cmd, v)
    else:
      if (len(self.m_config.in_names) != 1):
        self.infoNONE(f"Transporter has more than one inputs: {self.m_config.in_names}")
        exit(1)
      if (len(self.m_config.out_names) != 1):
        self.infoNONE(f"Transporter has more than one outputs: {self.m_config.out_names}")
        exit(1)
      self.m_transporter = transporter_module(self.m_my_name
                                             , path
                                             , useJson
                                             , self.m_config.in_names[0]
                                             , self.m_config.out_names[0]
                                             , self.m_config.delay
                                             , v)

  def write_buffer(self, outB, filename):
    self.infoDEBUG(f"Writing Output Buffer to {filename}.")
    if self.m_use_json:
      j = MessageToJson(outB
                       # , including_default_value_fields=False
                       , preserving_proto_field_name=True
                       )
      try:
        with open(filename, 'w', encoding='utf-8') as bffr:
          bffr.write(j)
      except Exception as e:
        self.infoNONE(f"Failed to write memOutput {filename}.")
        raise e
    else:
      try:
        with open(filename, 'wb', encoding='utf-8') as bffr:
          bffr.write(outB.SerializeToString())
      except Exception as e:
        self.infoNONE(f"Failed to write memOutput {filename}.")
        raise e

  def read_buffer(self, name, addrOffset):
    filename = self.m_path + "/mem/" + name + "_outBuf"
    filename+= ".json" if self.m_use_json else ".bin"
    self.infoDEBUG(f"read_buffer: Getting Buffer from {filename}.")
    try:
      with open(filename, 'rb') as file:
        mem_buffer = memBfr.buffer()
        string = file.read()
        if self.m_use_json:
          Parse(string, mem_buffer)
        else:
          mem_buffer.ParseFromString(string)

        # Sort the buffer in increasing address values
        mem_buffer.mem.sort(key=lambda x:x.address, reverse=False)

        # Add the offset to the address
        if addrOffset != 0:
          for idx in range(len(mem_buffer.mem)):
            mem_buffer.mem[idx].address += addrOffset

        self.infoDEBUG(f"read_buffer: {mem_buffer}")

        return mem_buffer
    except Exception as e:
      self.infoNONE(f"read_buffer: Failed to get {filename}.")
      raise e

  def consolidate_input_buf(self):
    ''' This function couples together all the memory buffers from the input transporters
    into the input memory buffer for this functional node.

    Note: The inBuffers from the inputs must not have conflicts, which is guaranteed by the transporters.
    (Addresses from each path have to fit in their own address space) 
    '''
    if (not self.m_config.is_transporter) and (len(self.m_config.in_names) != 0):
      # Name of the file which will be the result of this function
      outFile = self.m_path + "/mem/" + self.m_my_name + "_inBuf"
      outFile+= ".json" if self.m_use_json else ".bin"

      if len(self.m_config.in_names) == 1:
        inFile = self.m_path + "/mem/" + self.m_config.in_names[0] + "_outBuf"
        inFile+= ".json" if self.m_use_json else ".bin"
        self.infoMEDIUM(f"cons_in_buf: issuing copy command.")
        cmd = "cp " + inFile + " " + outFile
        self.execute_command(cmd)
      else:
        myInBuffer   = memBfr.buffer()

        '''
        Each edge has an individual set of address space, so we can collect them
        without having to use the offset to shift addresses.
        '''
        for name in self.m_config.in_names:
          myInBuffer.MergeFrom(self.read_buffer(name, 0))

        self.write_buffer(myInBuffer, outFile)
    else:
        self.infoDEBUG(f"#inNodeNames: {len(self.m_config.in_names)}, isTransporter: {self.m_config.is_transporter}.")
        raise SimException(f"Fail to gather input buffers")

  def distribute_output_buf(self):
    '''
    For distributing output buffer, all we need to do is to make copies of the file for 
    every transporter this process sending data to. Physical addresses in the outBuf file 
    are already designed for each transporter to realise its own address space. 
    '''
    if (not self.m_config.is_transporter) and (len(self.m_config.out_names) != 0):
      myOutFile = self.m_path + "/mem/" + self.m_my_name + "_outBuf"
      myOutFile+= ".json" if self.m_use_json else ".bin"
      for name in self.m_config.out_names:
        retFile = self.m_path + f"/mem/{name}_inBuf"
        retFile+= ".json" if self.m_use_json else ".bin"
        cmd = "cp " + myOutFile + " " + retFile
        self.execute_command(cmd)
      self.infoMEDIUM(f"dist_out_buf: issuing copy commands.")
    else:
      self.infoDEBUG(f"#outNodeNames: {len(self.m_config.out_names)}, isTransporter: {self.m_config.is_transporter}.")
      raise SimException(f"Fail to distribute output buffers")

  def doYourThing(self, globalTime):
    self.infoLOW(f"[@{globalTime}] Starting.")
    if not self.m_config.is_transporter: 
      if self.m_in_addrT != None:
        self.consolidate_input_buf()
        self.infoMEDIUM(f"[@{globalTime}] Trigger input address translator.")
        self.m_in_addrT.doYourThing(globalTime)

      self.infoMEDIUM(f"[@{globalTime}] Trigger process module.")
      self.m_process.doYourThing(globalTime)

      if self.m_out_addrT != None:
        self.infoMEDIUM(f"[@{globalTime}] Trigger input address translator.")
        self.m_out_addrT.doYourThing(globalTime)
        self.distribute_output_buf()
    else:
      self.infoMEDIUM(f"[@{globalTime}] Trigger Transporter.")
      self.m_transporter.doYourThing(globalTime)
    self.infoMEDIUM(f"Done.")


