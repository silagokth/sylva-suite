#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import json
import argparse
import sys
import random
import util

def conv_2d(image, kernel, stride):
    """
    2D convolution
    """
    H, W = image.shape
    KH, KW = kernel.shape
    S = stride

    OH = (H - KH) // S + 1
    OW = (W - KW) // S + 1

    conv_image = np.zeros((OH, OW))

    for h in range(OH):
        for w in range(OW):
            h_start = h * S
            h_end = h_start + KH
            w_start = w * S
            w_end = w_start + KW
            patch = image[h_start:h_end, w_start:w_end]
            conv_image[h, w] = np.tanh(np.sum(patch * kernel))
    return conv_image

def conv(image, kernel, bias, stride):
    output_image = []
    for i in range(kernel.shape[0]):
        kernel_2d = kernel[i]
        output_image_2d = []
        for i in range(image.shape[0]):
            input_image_2d = image[i]
            if i == 0:
                output_image_2d = conv_2d(input_image_2d, kernel_2d, stride)
            else:
                output_image_2d += conv_2d(input_image_2d, kernel_2d, stride)
        output_image.append(output_image_2d)
    output_image = np.array(output_image)
    assert output_image.shape == bias.shape
    output_image += bias
    return output_image

def read_input(memory, input_image_channel, input_image_size):
    storage_row_per_image_row = (input_image_size + 15) // 16
    current_row = 0
    input_image = []
    for c in range(input_image_channel):
        input_image.append([])
        for i in range(input_image_size):
            input_image[-1].append([])
            image_row = []
            for j in range(storage_row_per_image_row):
                image_row.extend(memory[current_row])
                current_row += 1
            input_image[-1][-1].extend(image_row[:input_image_size])
    
    return np.array(input_image)

def run(input_file, kernel_file, bias_file, output_file, input_image_channel, input_image_size, kernel_channel, kernel_size, stride):
    memory = util.json2fpmem(input_file)
    input_image = util.mem2mat(np.array(memory), (input_image_channel, input_image_size, input_image_size))
    memory = util.json2fpmem(kernel_file)
    kernel = util.mem2mat(np.array(memory), (kernel_channel, kernel_size, kernel_size))
    memory = util.json2fpmem(bias_file)
    output_image_size = (input_image_size - kernel_size) // stride + 1
    bias = util.mem2mat(np.array(memory), (kernel_channel, output_image_size, output_image_size))
    output_image = conv(input_image, kernel, bias, stride)
    out_mem = util.mat2mem(output_image)
    util.fpmem2json(out_mem, output_file)

def get_AP(input_image_channel, input_image_size, kernel_channel, kernel_size, stride, in_file, out_file):
    output_image_size = (input_image_size - kernel_size) // stride + 1
    output_image_channel = kernel_channel
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
    run_parser.add_argument('--kernel', type=str, help='Kernel file', required=True)
    run_parser.add_argument('--bias', type=str, help='Bias file', required=True)
    run_parser.add_argument('--input_image_channel', type=int, help='Input image channel', required=True)
    run_parser.add_argument('--input_image_size', type=int, help='Input image size', required=True)
    run_parser.add_argument('--kernel_channel', type=int, help='Kernel channel', required=True)
    run_parser.add_argument('--kernel_size', type=int, help='Kernel size', required=True)
    run_parser.add_argument('--stride', type=int, help='Stride', default=1)
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--input', type=str, help='Input address pattern', required=True)
    ap_parser.add_argument('--output', type=str, help='Output address pattern', required=True)
    ap_parser.add_argument('--input_image_channel', type=int, help='Input image channel', required=True)
    ap_parser.add_argument('--input_image_size', type=int, help='Input image size', required=True)
    ap_parser.add_argument('--kernel_channel', type=int, help='Kernel channel', required=True)
    ap_parser.add_argument('--kernel_size', type=int, help='Kernel size', required=True)
    ap_parser.add_argument('--stride', type=int, help='Stride', default=1)

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.in_mem, args.kernel, args.bias, args.out_mem, args.input_image_channel, args.input_image_size, args.kernel_channel, args.kernel_size, args.stride)
    elif args.subparser_name == 'ap':
        get_AP(args.input_image_channel, args.input_image_size, args.kernel_channel, args.kernel_size, args.stride, args.input, args.output)

if __name__ == '__main__':
    main()



