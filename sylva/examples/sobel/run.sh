#!/bin/sh
cc load.c -lcjson -o load
cc copy.c -lcjson -o copy
cc gx.c -lcjson -o gx
cc gy.c -lcjson -o gy
cc combine.c -lcjson -o combine
cc store.c -lcjson -o store

../../.venv/bin/python3 input.py input.png 
#./load global_mem_reference.json tmp_1.json
#./copy global_mem_reference.json tmp_1.json tmp_2.json
#./C global_mem_reference.json tmp_2.json
#rm -f tmp_*

