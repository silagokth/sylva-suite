#!/bin/sh
cc A.c -lcjson -o A
cc B.c -lcjson -o B
cc C.c -lcjson -o C
cp global_mem_image.json global_mem_reference.json
./A global_mem_reference.json tmp_1.json
./B global_mem_reference.json tmp_1.json tmp_2.json
./C global_mem_reference.json tmp_2.json
rm -f tmp_*

