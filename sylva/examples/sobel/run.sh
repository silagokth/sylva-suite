#!/bin/sh
# create a memory file for this image
../../.venv/bin/python3 model/input.py model/input.png mem/global_mem_image.json 
# compile python codes to executables
../../.venv/bin/pyinstaller model/load.py --name load --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/copy_node.py --name copy --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/gx.py --name gx --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/gy.py --name gy --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/combine.py --name combine --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 
../../.venv/bin/pyinstaller model/store.py --name store --distpath ./model/ --onefile --clean --specpath ./_spec/ --workpath ./_build/ 

#./load global_mem_reference.json tmp_1.json
#./copy global_mem_reference.json tmp_1.json tmp_2.json
#./C global_mem_reference.json tmp_2.json
#rm -f tmp_*

