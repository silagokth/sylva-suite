## @package addr_translator_base
#  Address translators, translates mem_buffers to
# mem_image and vice versa depending on if it is
# input or output addr_translator.

from lib.glic_sim.base_module import base_module
from lib.glic_sim.common import *

import lib.glic_sim.proto.addr_pattern_pb2 as addrPtrn
import lib.glic_sim.proto.translate_instr_pb2 as TransInst

from google.protobuf.json_format import MessageToJson, Parse, ParseDict

class addr_translator_base(base_module):

  def __init__(self, name, path, useJson, mType, v):
    super().__init__(name, mType, v)
    self.m_use_json = useJson
    self.m_path = path
    self.m_transTable = TransInst.translate_list()
    self.m_addrPtrn = addrPtrn.addr_pattern()
    self.infoDEBUG("init done.")

  def getAddressPattern(self):
    filename = self.m_path + "/" + self.m_my_name + "_"
    filename+= "in" if self.m_my_type == moduleType.inAddrTranslater else "out"
    filename+= "AP.json" #if self.m_use_json else "AP.bin"
    self.infoHIGH(f"Getting Address Pattern from {filename}.")
    try:
      with open(filename, 'rb') as ap:
        string = ap.read()
        if self.m_use_json:
          Parse(string, self.m_addrPtrn)
        else:
          self.m_addrPtrn.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {filename}.")
      raise e
    self.infoDEBUG(f"Address Patern:\n{self.m_addrPtrn}")

  def getTranslationTable(self):
    filename = self.m_path + "/" + self.m_my_name + "_"
    filename+= "in" if self.m_my_type == moduleType.inAddrTranslater else "out"
    filename+= "TT.json" #if self.m_use_json else "TT.bin"
    self.infoHIGH(f"Getting addr Translation Table from {filename}.")
    try:
      with open(filename, 'rb') as tt:
        string = tt.read()
        if self.m_use_json:
          Parse(string, self.m_transTable)
        else:
          self.m_transTable.ParseFromString(string)
    except Exception as e:
      self.infoNONE(f"Failed to get {filename}.")
      self.infoDEBUG(f"Parsed string:\n{string}")
      raise e
    self.infoDEBUG(f"Translation Table:\n{self.m_transTable}")
