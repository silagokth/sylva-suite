#!/usr/bin/env python
# -*- coding: utf-8 -*-

import argparse
import json
import numpy as np

# Constants
HEIGHT = 320
WIDTH = 320 

TOKEN_SIZE = 3200 # number of tokens required to support the whole image  
CHUNK_SIZE = 32 

MAX_ADDRESS = 4096
OFFSET = 0

def load_data(filename, size, offset):
    # read json file 
    with open(filename, 'r') as f:
        input_json = json.load(f)
    
    # create buffer 
    raw_data = bytearray((size + 1) * CHUNK_SIZE)   

    for entry in input_json["line"]:
        try:
            addr = int(entry["address"])
        except KeyError:
            addr = 0
        except Exception as e:
            raise ValueError
        if (addr-offset) < 0 or (addr-offset) >= size:
            continue
        try:
            hex_string = entry["value"]
        except KeyError:
            hex_string = ""
        except Exception as e:
            raise ValueError
        # padding 
        while (len(hex_string) < CHUNK_SIZE * 2):
            hex_string = "0" + hex_string
        bytes_chunk = bytes.fromhex(hex_string)[::-1]  # Convert and reverse LSB → MSB
        start = addr * CHUNK_SIZE
        end = start + len(bytes_chunk)
        raw_data[start:end] = bytes_chunk
    return raw_data

def write_data(data, size, filename):
    lines = []
    for i in range(0, len(data), CHUNK_SIZE):
        chunk = data[i:i+CHUNK_SIZE]
        reversed_chunk = chunk[::-1]  # back to LSB format
        hex_str = reversed_chunk.hex()
        while (len(hex_str) < CHUNK_SIZE * 2):
            hex_str = "00" + hex_str
        lines.append({
            "address": str(i // CHUNK_SIZE),
            "value": hex_str
        })
    
    # padding the rest with 0
    final_address = i // CHUNK_SIZE
    for i in range(final_address+1, (size + CHUNK_SIZE - 1) // CHUNK_SIZE):
        lines.append({
            "address": str(i),
            "value": "0" * CHUNK_SIZE
        })
 
    output = {"line": lines}
    with open(filename, "w") as f:
        json.dump(output, f, indent=2)

def main():
    ''' arguments '''
    parser = argparse.ArgumentParser(description="process node")

    parser.add_argument('--global-image', required=True, help="Path to global image file")
    parser.add_argument('--in-mem', required=False, help="Path to input memory JSON file")
    parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    args = parser.parse_args()

    ''' main '''
    input_data = load_data(args.global_image, MAX_ADDRESS, OFFSET)
    to_be_written = input_data[:TOKEN_SIZE * CHUNK_SIZE]
    write_data(to_be_written, TOKEN_SIZE * CHUNK_SIZE, args.out_mem)

if __name__ == '__main__':
    main()
