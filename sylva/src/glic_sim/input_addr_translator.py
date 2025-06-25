## @package input_addr_translator
#  Input Address translators, translates mem_buffers to
# mem_image.

import json
from lib.glic_sim.addr_translator_base import addr_translator_base
from lib.glic_sim.common import *

import lib.glic_sim.proto.mem_image_pb2 as memImg
import lib.glic_sim.proto.buffer_pb2 as memBfr
from google.protobuf.json_format import MessageToJson, Parse, ParseDict

class input_addr_translator(addr_translator_base):

  def __init__(self, name, path, useJson, v):
    super(input_addr_translator, self).__init__(name, path, useJson, moduleType.inAddrTranslater, v)
    self.m_input_buffer = memBfr.buffer()
    self.m_input_image  = memImg.mem_image()
    self.infoDEBUG("init done.")

  def writeInputMemImg(self):
    filename = self.m_path + "/mem/" + self.m_my_name + "_inMem"
    filename+= ".json" if self.m_use_json else ".bin"
    self.infoHIGH(f"Writing Memory Image to {filename}.")
    if self.m_use_json:
      j = MessageToJson(self.m_input_image
                       # , including_default_value_fields=False
                       , preserving_proto_field_name=True
                       )
      try:
        with open(filename, 'w', encoding='utf-8') as image:
          image.write(j)
      except Exception as e:
        self.infoNONE(f"Failed to write memImage {filename}.")
        raise e
    else:
      try:
        with open(filename, 'wb', encoding='utf-8') as image:
          image.write(self.m_input_image.SerializeToString())
      except Exception as e:
        self.infoNONE(f"Failed to write memImage {filename}.")
        raise e

  def getInputBuffer(self):
    filename = self.m_path + "/mem/" + self.m_my_name + "_inBuf"
    filename+= ".json" if self.m_use_json else ".bin"
    self.infoDEBUG(f"Getting Input Buffer from {filename}.")
    try:
      with open(filename, 'rb') as file:
        string = file.read()
        if self.m_use_json:
          Parse(string, self.m_input_buffer)
        else:
          self.m_input_buffer.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {filename}.")
      raise e

  def doYourThing(self, globalTime):
    self.infoMEDIUM(f"[@{globalTime}] Starting Addr Translation.")
    self.getTranslationTable()
    self.getAddressPattern()
    self.getInputBuffer()

    # Sort the buffer with earliest vales (lower cycle) on top
    self.m_input_buffer.mem.sort(key=lambda x:x.cycle, reverse=False)

    # make sure address patern is in increasing order of cycles.
    self.m_addrPtrn.addr_ptrn.sort(key=lambda x:x.cycle, reverse=False)
    self.infoDEBUG(f"Sorted Addr Ptrn:\n{self.m_addrPtrn.addr_ptrn}")

    refTime = globalTime 
    for i in self.m_addrPtrn.addr_ptrn:
      refTime = globalTime + i.cycle

      tr = next((x for x in self.m_transTable.list if (x.addr_in == i.address)), None) 
      if not bool(tr):
        self.infoNone(f"[@{refTime}] Failed to translate address for inAddr: 0x{i.address:x}")
        raise SimException(f"Fail to translate an address") 
    
      # +1 because we want to make sure that the data is completely written into the buffer
      lst_buf = [x for x in self.m_input_buffer.mem if ((x.address == tr.addr_out) and 
                                                        ((x.cycle+1) <= refTime))]
      bfr = lst_buf[0] if len(lst_buf) else None
 
      if len(lst_buf) == 1:
        self.m_input_buffer.mem.remove(bfr)
        self.infoHIGH(f"[@{refTime}] For inAddr 0x{i.address:x} found value {bfr.value}.")
      elif len(lst_buf) > 1:
        self.infoNONE(f"[@{refTime}] For inAddr 0x{i.address:x} write collision detected {lst_buf}")
        raise SimException(f"Fail to get a value from the buffer")
      else:
        self.infoNONE(f"[@{refTime}] For inAddr 0x{i.address:x} could NOT find a value.")
        raise SimException(f"Fail to get a value from the buffer")

      # Add a new memory line to memoryImage
      imgLine = self.m_input_image.line.add()
      imgLine.address = i.address
      imgLine.value = bfr.value if bool(bfr) else "0"
      self.infoDEBUG(f" Added {imgLine.address:x}: {imgLine.value}")
 
    self.m_input_image.line.sort(key=lambda x:x.address, reverse=False)
    self.writeInputMemImg()
    self.infoLOW(f"[@{refTime}] Done.")
