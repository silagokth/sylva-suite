## @package process_module
#  This is the functionality class. It is assumed to run
#  in zero time and would run a shell command.

from lib.glic_sim.base_module import base_module
from lib.glic_sim.common import *

class process_module(base_module):

  def __init__(self, name, path, cmd, v):
    super(process_module, self).__init__(name, moduleType.process, v)
    self.m_command = cmd
    self.cmd_path = path
    self.infoDEBUG("init done.")

  def doYourThing(self, globalTime):
    self.infoLOW(f"[@{globalTime}] preparing to run: {self.m_command}")
    res, err = self.execute_command(self.m_command)
    self.infoLOW(f"Process output = {res} (err={err})")
    self.infoLOW("Process done.")
