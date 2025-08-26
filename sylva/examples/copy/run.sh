#!/bin/sh

# Check if the token argument is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <token>"
  exit 1
fi

TOKEN=$1

cc A.c -lcjson -o A
cc B.c -lcjson -o B
cc C.c -lcjson -o C

cp mem/global_mem_image.json mem/global_mem_reference.json
./A --global-image mem/global_mem_image.json --out-mem _A.json --token $TOKEN
./B --in-mem _A.json --out-mem _B.json
./C --global-image mem/global_mem_reference.json --in-mem _B.json --token $TOKEN

