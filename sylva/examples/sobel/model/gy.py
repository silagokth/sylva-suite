#!/usr/bin/env python
# -*- coding: utf-8 -*-

import argparse
import json
import numpy as np

# Constants
HEIGHT = 320
WIDTH = 320 

TOKEN_SIZE = 3200 # number of tokens required to support the whole image  
OUT_TOKEN_SIZE = 3200 # scaling up for output 
CHUNK_SIZE = 32

MAX_ADDRESS = 4096
TOTAL_MEMORY = 4096 * 6 # Total 6 nodes 
OFFSET = 4096 * 3

def load_all(filename):
    # read json file 
    with open(filename, 'r') as f:
        input_json = json.load(f)
    
    memory_blocks = {}
    for entry in input_json["line"]:
        try:
            addr = int(entry["address"])
        except KeyError:
            addr = 0
        except Exception as e:
            raise ValueError
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
    if len(data) < HEIGHT * WIDTH:
        raise ValueError("Insufficient pixel data for declared dimensions.")

    pixel_data = data[:HEIGHT * WIDTH]
    img = np.frombuffer(pixel_data, dtype=np.uint8).reshape((HEIGHT, WIDTH))
    return img

def gy(img):
    '''Gy convolution for channel 1'''
    kernel = np.array([[1, 2, 1], [0, 0, 0], [-1, -2, -1]]
                      )  # Sobel kernel for Gy
    gy = np.array([[0 for j in range(img.shape[1])]
                  for i in range(img.shape[0])], dtype=np.int32)
    for i in range(1, img.shape[0] - 1):
        for j in range(1, img.shape[1] - 1):
            gy[i][j] = np.sum(np.multiply(
                img[i - 1:i + 2, j - 1:j + 2], kernel))
    return gy

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
    image = decode_image(input_data)
    gy_image = gy(image)
    gy_data = encode_image(gy_image)
    write_data(gy_data, TOKEN_SIZE * CHUNK_SIZE, args.out_mem)

if __name__ == '__main__':
    main()
