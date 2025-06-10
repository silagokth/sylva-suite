#! /usr/bin/python3

## @package base_module
#  This will be the base class encapsulating common functions.
#

import os
import subprocess
import shlex
from lib.glic_sim.common import *

# The base module
class base_module:
  cmd_path = None
  cstm_env = None # @TODO: remove

  def __init__(self, name, ty, v):
    assert type(ty) == type(moduleType.uninit)
    self.m_my_name = name
    self.m_my_type = ty
    self.m_my_verbosity = v

  #set verbosity for this module
  def set_verbosity(self, ver):
    assert type(ver) == type(self.m_my_verbosity)
    self.m_my_verbosity = ver

  # def line(self):
  #   frame = getframeinfo(currentframe())
  #   return frame.lineno

  # Function to report verbosity controlled info
  def info(self, ver, *msg):
    assert type(ver) == type(self.m_my_verbosity)
    if(ver <= self.m_my_verbosity):
      print(self.m_my_name+"["+self.m_my_type.name+"]"+":", *msg)

  # Convinience
  def infoNONE(self, *msg):
    self.info(verbosity.NONE, *msg)

  def infoLOW(self, *msg):
    self.info(verbosity.LOW, *msg)

  def infoMEDIUM(self, *msg):
    self.info(verbosity.MEDIUM, *msg)

  def infoHIGH(self, *msg):
    self.info(verbosity.HIGH, *msg)

  def infoFULL(self, *msg):
    self.info(verbosity.FULL, *msg)

  def infoDEBUG(self, *msg):
    self.info(verbosity.DEBUG, *msg)

  # Run subprocesses
  def execute_command(self, cmd, _shell=False):
    self.infoFULL('execute_command: CMD = ', cmd)
    if isinstance(cmd, str) and _shell != True:
      _cmd = shlex.split(cmd)

    # We must expand any variables for non-shell mode.
    # Shell mode expands variables in the command
    if _shell != True:
      cmd = [os.path.expandvars(arg) for arg in _cmd]
    self.infoDEBUG('split command = ', cmd)
    popen_arg_list = {
                        "shell" : False,
                        "stdout": subprocess.PIPE,
                        "stderr": subprocess.PIPE,
                        "shell" : _shell,
                        "env"   : os.environ.copy()
                      }

    if bool(self.cmd_path):
      popen_arg_list["cwd"] = self.cmd_path

    p =  subprocess.Popen(cmd, **popen_arg_list)
    outs, errs = p.communicate()
    self.infoDEBUG(' OUTPUT = ', outs)
    if errs:
      self.infoDEBUG(' ERROR = ', errs)
    return outs, errs

  def doYourThing(self, globalTime):
    self.infoNONE("THIS CANNOT BE UNIMPLEMENTED IN A MODULE!!!")
    assert False

if __name__ == "__main__":
  base_inst = base_module("test_baseModule", moduleType.uninit, verbosity.DEBUG)

  base_inst.cmd_path = "/home/drake/Repos/GLICsim/tb"
  cmd = "$MY_PY3 $GLICREPO/bin/matrixOper.py -h"
  base_inst.execute_command(cmd)
