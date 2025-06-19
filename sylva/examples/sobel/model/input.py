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

def convert_to_grayscale(img):
    gray = np.array([[0.299 * img[i][j][0] + 0.587 *
                      img[i][j][1] + 0.114 * img[i][j][2] for j in range(img.shape[1])]
                     for i in range(img.shape[0])])
    return gray.astype(np.uint8)

def encode_image(img):
    height, width = img.shape
    if (height != HEIGHT or width != WIDTH):
        raise ValueError('image size should be 320x320')
    image_data = img.tobytes()
    return image_data  # Total binary content

def main():
    '''main function'''
    if (len(sys.argv) < 3):
        print("Usage: ./program image_file.png output.json")
    img = cv2.imread(sys.argv[1])
    if img is None:
        raise ValueError('fail to read image')
    
    gray = convert_to_grayscale(img)
    full_data = encode_image(gray)

    lines = []
    for i in range(0, len(full_data), CHUNK_SIZE):
        chunk = full_data[i:i+CHUNK_SIZE]
        reversed_chunk = chunk[::-1]
        hex_string = reversed_chunk.hex()
        while (len(hex_string) < CHUNK_SIZE * 2):
            hex_string = "00" + hex_string
        lines.append({
            "address": str(i // CHUNK_SIZE),
            "value": hex_string
        })
    
    final_address = i // CHUNK_SIZE
    if final_address >= MAX_ADDRESS:
        raise ValueError("image is too large than the address space specified")
    if final_address >= TOKEN_SIZE:
        raise ValueError("image is too large for transferring through the communication channel")

    # padding the rest with 0
    for i in range(final_address+1, TOTAL_MEMORY):
        lines.append({
            "address": str(i),
            "value": "00" * CHUNK_SIZE
        })
    output = {"line": lines}
    
    with open(sys.argv[2], "w") as f:
        json.dump(output, f, indent=2)

if __name__ == '__main__':
    main()
