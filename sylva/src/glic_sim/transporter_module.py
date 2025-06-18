## @package transporter_module
#  This module moves the data from one not to the other.

import json

from lib.glic_sim.base_module import base_module
from lib.glic_sim.common import *

import lib.glic_sim.proto.transport_inst_pb2 as TransInst
import lib.glic_sim.proto.buffer_pb2 as memBfr
from google.protobuf.json_format import MessageToJson, Parse, ParseDict

#TODO: support multiple firing times

class transporter_module(base_module):

  def __init__(self, name, path, useJson, in_name, out_name, delay, v):
    super(transporter_module, self).__init__(name, moduleType.transporter, v)
    self.m_path         = path
    self.m_in_name      = in_name
    self.m_out_name     = out_name
    self.m_use_json     = useJson
    self.m_delay        = delay
    self.m_in_buf_name  = self.m_path + "/mem/" + self.m_my_name + "_inBuf"
    self.m_out_buf_name = self.m_path + "/mem/" + self.m_my_name + "_outBuf"
    self.m_in_buf_name += ".json" if self.m_use_json else ".bin"
    self.m_out_buf_name+= ".json" if self.m_use_json else ".bin"
    self.m_inst         = TransInst.transport_list()
    self.m_in_buf       = memBfr.buffer()
    self.infoDEBUG("init done.")

  def getTransportInst(self):
    filename = self.m_path + "/" + self.m_my_name + "_TransInst"
    filename+= ".json"# if self.m_use_json else ".bin"
    self.infoDEBUG(f"Getting the instruction list from {filename}")
    try:
      with open(filename, 'rb') as ap:
        string = ap.read()
        if self.m_use_json:
          Parse(string, self.m_inst)
        else:
          self.m_inst.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {filename}.")
      raise e

  def getInputBuffer(self):
    self.infoDEBUG(f"Getting Input Buffer from {self.m_in_buf_name}.")
    try:
      with open(self.m_in_buf_name, 'rb') as file:
        string = file.read()
        if self.m_use_json:
          Parse(string, self.m_in_buf)
        else:
          self.m_in_buf.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {self.m_in_buf_name}.")
      raise e

  def doYourThing(self, globalTime):
    self.infoMEDIUM(f"[@{globalTime}] Transfring data from {self.m_in_name} to {self.m_out_name}")
    self.getTransportInst()
    self.getInputBuffer()

    # Sort the buffer with earliest values (lower cycle) on top.
    self.m_in_buf.mem.sort(key=lambda x: x.cycle, reverse=False)

    # We want to make sure the instruction is ordered by cycle time.
    self.m_inst.inst_list.sort(key=lambda x: x.cycle, reverse=False)

    self.infoDEBUG(f"Sorted Instruction list:\n{self.m_inst.inst_list}")
    self.m_out_buf = memBfr.buffer()

    refTime = globalTime
    for i in self.m_inst.inst_list:
      refTime = globalTime + i.cycle 
      # +1 because we want to make sure that the data is completely written into the buffer
      lst_buf = [x for x in self.m_in_buf.mem if ((x.address == i.addr_rd) and ((x.cycle+1) <= refTime))]
      bfr = lst_buf[0] if len(lst_buf) else None

      if len(lst_buf) == 1:
        self.m_in_buf.mem.remove(bfr)
        self.infoHIGH(f"[@{refTime}] For inAddr 0x{i.addr_rd:x} found value {bfr.value}.")
      elif len(lst_buf) > 1:
        self.infoNONE(f"[@{refTime}] For inAddr 0x{i.addr_rd:x} write collision detected")
        raise SimException(f"Fail to get a value from the buffer")
      else:
        self.infoNONE(f"[@{refTime}] For inAddr 0x{i.addr_rd:x} could NOT find a value.")
        raise SimException(f"Fail to get a value from the buffer")

      line = self.m_out_buf.mem.add()
      line.cycle = refTime + self.m_delay 
      line.address = i.addr_wr
      line.value = bfr.value if bool(bfr) else "0"

    self.infoDEBUG(f"outbuffer = {self.m_out_buf}")
    self.infoMEDIUM(f"[@{refTime}] Writing output buffer {self.m_out_name}")
    if self.m_use_json:
      j = MessageToJson(self.m_out_buf
                       # , including_default_value_fields=False
                       , preserving_proto_field_name=True
                       )
      try:
        with open(self.m_out_buf_name, 'w', encoding='utf-8') as image:
          # json.dump(j, image, ensure_ascii=True, indent=2)
          image.write(j)
      except Exception as e:
        self.infoNONE(f"Failed to write memOutput {self.m_out_buf_name}.")
        raise e
    else:
      try:
        with open(self.m_out_buf_name, 'wb', encoding='utf-8') as image:
          image.write(self.m_output_buffer.SerializeToString())
      except Exception as e:
        self.infoNONE(f"Failed to write memOutput {self.m_out_buf_name}.")
        raise e
