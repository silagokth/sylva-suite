## @package node
#  This is a node. It contains process & addrTranslators.
# Depending on the config it can have either both input
# and output addrTranslators or one or the other.
import json

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
      if len(self.m_config.in_names) != 0:
        self.m_in_addrT = input_addr_translator(self.m_my_name, path, useJson, v)
      if len(self.m_config.out_names) != 0:
        self.m_out_addrT = output_addr_translator(self.m_my_name, path, useJson, v)
      self.m_process = process_module(self.m_my_name, self.m_config.process_cmd, v)
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
    into the input memory buffer for this functional node

    Note: The inBuffers from the inputs must not have conflicts, it is always assumed to
    have a continuous address range from 0 - some value. The order of the input list will determine
    which buffer should be read first and the (maxAddress + 1) becomes the offset that gets
    added to the next buffer that is read. The offsets are cumulative.

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

        '''In the order of inputNode names in the in_name list, the output address from a node
        is offset by the highest address + 1 from the node before.
        '''
        addrOffset = 0
        for name in self.m_config.in_names:
          myInBuffer.MergeFrom(self.read_buffer(name, addrOffset))
          addrOffset = myInBuffer.mem[-1].address
          self.infoHIGH(f"#BufLines: {len(myInBuffer.mem)}, newOffset: 0x{addrOffset:x}.")

        self.write_buffer(myInBuffer, outFile)
    else:
        self.infoDEBUG(f"#inNodeNames: {len(self.m_config.in_names)}, isTransporter: {self.m_config.is_transporter}.")

  def doYourThing(self, globalTime):
    self.infoLOW(f"[@{globalTime}] Starting.")
    if not self.m_config.is_transporter:
      self.consolidate_input_buf()
      if self.m_in_addrT != None:
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
    self.infoMEDIUM(f"[@{globalTime}] Done.")

  def distribute_output_buf(self):
    if (not self.m_config.is_transporter) and (len(self.m_config.out_names) != 0):
      if len(self.m_config.out_names) == 1:
        myOutFile = self.m_path + "/mem/" + self.m_my_name + "_outBuf"
        myOutFile+= ".json" if self.m_use_json else ".bin"
        retFile = self.m_path + "/mem/" + self.m_config.out_names[0] + "_inBuf"
        retFile+= ".json" if self.m_use_json else ".bin"
        self.infoMEDIUM(f"dist_out_buf: issuing copy command.")
        cmd = "cp " + myOutFile + " " + retFile
        self.execute_command(cmd)
      else:
        startAddr = 0
        outBuffer = memBfr.buffer()
        outBuffer.CopyFrom(self.read_buffer(self.m_my_name, 0))

        for name in self.m_config.out_names:
          retFile = self.m_path + "/mem/" + name + "_inBuf"
          retFile+= ".json" if self.m_use_json else ".bin"
          if bool(self.m_config.tokenSize.get(name)):
            # if the tokenSize value exist trim the output and write the new file
            ret = memBfr.buffer()
            size = self.m_config.tokenSize.get(name)
            self.infoDEBUG(f"dist_out_buf: starting address is 0x{startAddr}.")
            self.infoHIGH(f"dist_out_buf: tokenSize[{name}] = {size}.")
            self.infoHIGH(f"Fetching addr 0x{startAddr:x} through 0x{(startAddr+size):x}.")

            fL = [msg for msg in outBuffer.mem if msg.address >= startAddr and msg.address < (startAddr+size)]
            ret.mem.extend(fL)

            # Addresses must be moved to start from 0
            for idx in range(len(ret.mem)):
              ret.mem[idx].address -= startAddr

            # Next time start from the last address from this iteration.
            startAddr += size
            self.write_buffer(ret, retFile)
          else:
            # @TODO:
            self.infoDEBUG(f"dist_out_buf: tokenSize[{name}] not find.")

