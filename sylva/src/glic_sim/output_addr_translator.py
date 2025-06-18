## @package output_addr_translator
#  Input Address translators, translates mem_buffers to
# mem_image.

import json
from lib.glic_sim.addr_translator_base import addr_translator_base
from lib.glic_sim.common import *

import lib.glic_sim.proto.mem_image_pb2 as memImg
import lib.glic_sim.proto.buffer_pb2 as memBfr
from google.protobuf.json_format import MessageToJson, Parse, ParseDict

class output_addr_translator(addr_translator_base):

  def __init__(self, name, path, useJson, v):
    super(output_addr_translator, self).__init__(name, path, useJson, moduleType.outAddrTranslater, v)
    self.m_output_buffer = memBfr.buffer()
    self.m_output_image  = memImg.mem_image()
    self.infoDEBUG("init done.")
 
  def writeOutputBfr(self):
    filename = self.m_path + "/mem/" + self.m_my_name + "_outBuf"
    filename+= ".json" if self.m_use_json else ".bin"
    self.infoDEBUG(f"Writing Output Buffer to {filename}.")
    if self.m_use_json:
      j = MessageToJson(self.m_output_buffer
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
          bffr.write(self.m_output_buffer.SerializeToString())
      except Exception as e:
        self.infoNONE(f"Failed to write memOutput {filename}.")
        raise e

  def getOutputImage(self):
    filename = self.m_path + "/mem/" + self.m_my_name + "_outMem"
    filename+= ".json" if self.m_use_json else ".bin"
    self.infoDEBUG(f"Getting Input Memory from {filename}.")
    try:
      with open(filename, 'rb') as file:
        string = file.read()
        if self.m_use_json:
          Parse(string, self.m_output_image)
        else:
          self.m_output_image.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {filename}.")
      raise e

  def doYourThing(self, globalTime):
    self.infoMEDIUM(f"[@{globalTime}] Starting Addr Translation.")
    self.getTranslationTable()
    self.getAddressPattern()
    
    self.getOutputImage()

    # make sure address patern is in increasing order of cycles.
    self.m_addrPtrn.addr_ptrn.sort(key=lambda x:x.cycle, reverse=False)
    self.infoDEBUG(f"Sorted Addr Ptrn:\n{self.m_addrPtrn.addr_ptrn}")

    refTime = globalTime
    for i in self.m_addrPtrn.addr_ptrn:
      refTime = globalTime + i.cycle
      ImgLine = next((x for x in self.m_output_image.line if (x.address == i.address)), None)
      if bool(ImgLine):
        self.infoHIGH(f"[@{refTime}] addr 0x{i.address:x} found value {ImgLine.value}.")
      else:
        self.infoNone(f"[@{refTime}] addr 0x{i.address:x} not found.")
        raise SimException(f"Fail to get a value from the memory") 

      tr = next((x for x in self.m_transTable.list if (x.addr_in == i.address)), None)

      if not bool(tr):
        self.infoNone(f"[@{refTime}] Failed to translate address for outAddr: 0x{i.address:x}")
        raise SimException(f"Fail to translate an address") 

      # Add a new Buffer line to output buffer
      bfrLine = self.m_output_buffer.mem.add()
      bfrLine.cycle = refTime 
      bfrLine.value = ImgLine.value if bool(ImgLine) else "0"
      bfrLine.address = tr.addr_out if bool(tr) else i.address
      self.infoDEBUG(f" Added @{bfrLine.cycle} cycle 0x{bfrLine.address:x}: {bfrLine.value}")

    self.writeOutputBfr()
    self.infoLOW(f"[@{refTime}] Done.")
