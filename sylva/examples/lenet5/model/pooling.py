#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import json
import argparse
import sys
import random
import util

def pooling_2d(image, kernel_size, stride, mode):
    """
    2D pooling

    Parameters:
    - image: 2D numpy array
    - kernel_size: int
    - stride: int
    - mode: str, 'max' or 'average'

    Returns:
    - pooled_image: 2D numpy array
    """
    H, W = image.shape
    KH, KW = kernel_size, kernel_size
    S = stride

    OH = (H - KH) // S + 1
    OW = (W - KW) // S + 1

    pooled_image = np.zeros((OH, OW))

    for h in range(OH):
        for w in range(OW):
            h_start = h * S
            h_end = h_start + KH
            w_start = w * S
            w_end = w_start + KW
            patch = image[h_start:h_end, w_start:w_end]
            if mode == 'max':
                pooled_image[h, w] = np.tanh(np.max(patch))
            elif mode == 'average':
                pooled_image[h, w] = np.tanh(np.mean(patch))

    return pooled_image

def pooling(image, kernel_size, stride, mode):
    output_image = []
    for i in range(image.shape[0]):
        output_image.append(pooling_2d(image[i], kernel_size, stride, mode))
    return np.array(output_image)

def run(input_file, output_file, input_image_channel, input_image_size, kernel_size, stride, mode):
    memory = util.json2fpmem(input_file)
    input_image = util.mem2mat(np.array(memory), (input_image_channel, input_image_size, input_image_size))
    output_image = pooling(input_image, kernel_size, stride, mode)
    memory = util.mat2mem(output_image)
    util.fpmem2json(memory, output_file)

def get_AP(input_image_channel, input_image_size, kernel_size, stride, in_file, out_file):
    output_image_size = (input_image_size - kernel_size) // stride + 1
    output_image_channel = input_image_channel
    input_storage_row_per_image_row = (input_image_size + 15) // 16
    output_storage_row_per_image_row = (output_image_size + 15) // 16
    in_size = input_image_channel * input_image_size * input_storage_row_per_image_row
    out_size = output_image_channel * output_image_size * output_storage_row_per_image_row
    util.gen_addr_pattern(in_size, in_file)
    util.gen_addr_pattern(out_size, out_file)

def main():
    parser = argparse.ArgumentParser(description='Exec')

    subparsers = parser.add_subparsers(dest="subparser_name")
    run_parser = subparsers.add_parser('run', help='Run function')
    run_parser.add_argument('--global-image', required=False, help="Path to global image file")
    run_parser.add_argument('--in-mem', required=True, help="Path to input memory JSON file")
    run_parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    run_parser.add_argument('--input_image_channel', type=int, help='Input image channel', required=True)
    run_parser.add_argument('--input_image_size', type=int, help='Input image size', required=True)
    run_parser.add_argument('--kernel_size', type=int, help='Kernel size', required=True)
    run_parser.add_argument('--stride', type=int, help='Stride', default=1)
    run_parser.add_argument('--mode', type=str, help='Mode', default='average')
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--input', type=str, help='Input address pattern', required=True)
    ap_parser.add_argument('--output', type=str, help='Output address pattern', required=True)
    ap_parser.add_argument('--input_image_channel', type=int, help='Input image channel', required=True)
    ap_parser.add_argument('--input_image_size', type=int, help='Input image size', required=True)
    ap_parser.add_argument('--kernel_size', type=int, help='Kernel size', required=True)
    ap_parser.add_argument('--stride', type=int, help='Stride', default=1)

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.in_mem, args.out_mem, args.input_image_channel, args.input_image_size, args.kernel_size, args.stride, args.mode)
    elif args.subparser_name == 'ap':
        get_AP(args.input_image_channel, args.input_image_size, args.kernel_size, args.stride, args.input, args.output)

if __name__ == '__main__':
    main()



