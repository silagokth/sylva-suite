## @package common
#  This is to house common defines.

from enum import IntEnum, Enum # for using enumeration.
from inspect import currentframe, getframeinfo

import lib.glic_sim.const as const

class SimException(Exception):
  def __init__(self, value):
    self.parameter = value

  def __str__(self):
    return repr(self.parameter)

# Enumeration for verbosity
class verbosity(IntEnum):
  NONE   = 0
  LOW    = 100
  MEDIUM = 200
  HIGH   = 300
  FULL   = 400
  DEBUG  = 500

# Shorthand Constants
PY_NONE   = verbosity.NONE
PY_LOW    = verbosity.LOW
PY_MEDIUM = verbosity.MEDIUM
PY_HIGH   = verbosity.HIGH
PY_FULL   = verbosity.FULL
PY_DEBUG  = verbosity.DEBUG

# Enumeration for the type of module
class moduleType(Enum):
  uninit            = 0
  inAddrTranslater  = 100
  process           = 200
  outAddrTranslater = 300
  transporter       = 400
  node              = 500
  TopModule         = 600
