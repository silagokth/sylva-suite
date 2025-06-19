#!/usr/bin/env python
# -*- coding: utf-8 -*-

import argparse
import json
import numpy as np

# Constants
HEIGHT = 320
WIDTH = 320 

IN_TOKEN_SIZE = 4 * 3200 # INT pixel 
OUT_TOKEN_SIZE = 3200 # number of tokens required to support the whole image  
CHUNK_SIZE = 32

MAX_ADDRESS = 4096
TOTAL_MEMORY = 4096 * 6 # Total 6 nodes 
OFFSET = 4096 * 4

def load_all(filename):
    # read json file 
    with open(filename, 'r') as f:
        input_json = json.load(f)
    
    memory_blocks = {}
    for entry in input_json["line"]:
        addr = int(entry["address"])
        hex_string = entry["value"]        
        # padding 
        while (len(hex_string) < CHUNK_SIZE * 2):
            hex_string = "0" + hex_string
        bytes_chunk = bytes.fromhex(hex_string)[::-1]  # Convert and reverse LSB → MSB
        memory_blocks[addr] = bytes_chunk
    
    max_addr = max(memory_blocks.keys())
    raw_data = bytearray((max_addr + 1) * CHUNK_SIZE)
    for addr, chunk in memory_blocks.items():
        start = addr * CHUNK_SIZE
        end = start + CHUNK_SIZE
        raw_data[start:end] = chunk
    return raw_data

def encode_image(img):
    height, width = img.shape
    if (height != HEIGHT or width != WIDTH):
        raise ValueError('image size should be 320x320')
    image_data = img.tobytes()
    return image_data  # Total binary content

def decode_image(data):
    if len(data) < (4 * HEIGHT * WIDTH):
        raise ValueError("Insufficient pixel data for declared dimensions.")

    pixel_data = data[:4 * HEIGHT * WIDTH]
    img = np.frombuffer(pixel_data, dtype=np.int32).reshape((HEIGHT, WIDTH))
    return img

def combine(gx, gy):
    '''combine Gx and Gy results to get final result'''
    gx = gx.astype(np.int32)
    gy = gy.astype(np.int32)
    result = np.sqrt(np.square(gx) + np.square(gy))
    result = result.astype(np.uint8)
    return result

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

    parser.add_argument('--global-image', required=False, help="Path to global image file")
    parser.add_argument('--in-mem', required=True, help="Path to input memory JSON file")
    parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    args = parser.parse_args()

    ''' main '''
    input_data = load_all(args.in_mem)
    raw_data1, raw_data2 = input_data[IN_TOKEN_SIZE * CHUNK_SIZE:], input_data[:IN_TOKEN_SIZE * CHUNK_SIZE]
    image1, image2 = decode_image(raw_data1), decode_image(raw_data2)
    combine_image = combine(image1, image2)
    combine_data = encode_image(combine_image)
    write_data(combine_data, OUT_TOKEN_SIZE * CHUNK_SIZE, args.out_mem)

if __name__ == '__main__':
    main()
