#!/usr/bin/env python
# -*- coding: utf-8 -*-
import sys
import cv2
import numpy as np
import json

HEIGHT = 320
WIDTH = 320 

TOKEN_SIZE = 3200 # number of tokens required to support the whole image  
CHUNK_SIZE = 32

MAX_ADDRESS = 4096
TOTAL_MEMORY = 4096 * 6 # Total 6 nodes 
OFFSET_STORE = 4096 * 5

def load_data(filename, size, offset):
    # read json file 
    with open(filename, 'r') as f:
        input_json = json.load(f)
    
    # create buffer 
    raw_data = bytearray((size + 1) * CHUNK_SIZE)   

    for entry in input_json["line"]:
        addr = int(entry["address"])
        if (addr-offset) < 0 or (addr-offset) >= size:
            continue
        hex_string = entry["value"]        
        # padding 
        while (len(hex_string) < CHUNK_SIZE * 2):
            hex_string = "0" + hex_string
        bytes_chunk = bytes.fromhex(hex_string)[::-1]  # Convert and reverse LSB → MSB
        start = (addr - offset) * CHUNK_SIZE
        end = start + len(bytes_chunk)
        raw_data[start:end] = bytes_chunk
    return raw_data

def decode_image(data):
    if len(data) < HEIGHT * WIDTH:
        raise ValueError("Insufficient pixel data for declared dimensions.")

    pixel_data = data[:HEIGHT * WIDTH]
    img = np.frombuffer(pixel_data, dtype=np.uint8).reshape((HEIGHT, WIDTH))
    return img

def main():
    '''main function'''
    if (len(sys.argv) < 3):
        print("Usage: ./program global_mem_output.json result.png")
    input_data = load_data(sys.argv[1], MAX_ADDRESS, OFFSET_STORE)
    image = decode_image(input_data)
    cv2.imwrite(sys.argv[2], image.astype(np.uint8))

if __name__ == '__main__':
    main()
