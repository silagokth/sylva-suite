#!/bin/sh
# compile python codes to executables
../../.venv/bin/pyinstaller model/load_input.py --name load_input --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/conv.py --name conv --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/pooling.py --name pooling --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/fc.py --name fc --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/reshape.py --name reshape --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/store_output.py --name store_output --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 


