#!/usr/bin/env python3
import mosaic_sdk

import sys
from pathlib import Path # if you haven't already done so
file = Path(__file__).resolve()
parent, root = file.parent, file.parents[1]
sys.path.append(str(root))

# Additionally remove the current file's directory from sys.path
try:
    sys.path.remove(str(parent))
except ValueError:
    pass

from python_sdk import client

__doc__ = mosaic_sdk.__doc__
if hasattr(mosaic_sdk, "__all__"):
	__all__ = mosaic_sdk.__all__
	print(__all__)


# client = mosaic_sdk.Client("[::]:8080")
pyclient = client.PyClient()
