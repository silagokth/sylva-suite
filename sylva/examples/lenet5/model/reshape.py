#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import json
import argparse
import sys
import random
import util

def reshape(input_matrix, input_row, input_col, output_row, output_col):
    """
    Reshape input matrix to output matrix

    Parameters:
    - input_matrix: 2D numpy array
    - input_row: int
    - input_col: int
    - output_row: int
    - output_col: int

    Returns:
    - None
    """
    # check size match
    assert input_row * input_col == output_row * output_col

    output_matrix = np.zeros((output_row, output_col))

    for i in range(output_row):
        for j in range(output_col):
            abs_idx = i*output_col + j
            input_i = abs_idx // input_col
            input_j = abs_idx % input_col
            output_matrix[i][j] = input_matrix[input_i][input_j]

    return output_matrix

def run(input_file, input_row, input_col, output_file, output_row, output_col):
    memory = util.json2fpmem(input_file)
    input_image = util.mem2mat(np.array(memory), (input_row, input_col))
    output_image = reshape(input_image, input_row, input_col, output_row, output_col)
    out_mem = util.mat2mem(output_image)
    util.fpmem2json(out_mem, output_file)

def get_AP(input_row, input_col, output_row, output_col, in_file, out_file):
    input_storage_row_per_image_row = (input_col + 15) // 16
    output_storage_row_per_image_row = (output_col + 15) // 16
    in_size = input_storage_row_per_image_row * input_row
    out_size = output_storage_row_per_image_row * output_row
    util.gen_addr_pattern(in_size, in_file)
    util.gen_addr_pattern(out_size, out_file)

def main():
    parser = argparse.ArgumentParser(description='Exec')

    subparsers = parser.add_subparsers(dest="subparser_name")
    run_parser = subparsers.add_parser('run', help='Run function')
    run_parser.add_argument('--global-image', required=False, help="Path to global image file")
    run_parser.add_argument('--in-mem', required=True, help="Path to input memory JSON file")
    run_parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    run_parser.add_argument('--input_row', type=int, help='Input row', required=True)
    run_parser.add_argument('--input_col', type=int, help='Input col', required=True)
    run_parser.add_argument('--output_row', type=int, help='Output row', required=True)
    run_parser.add_argument('--output_col', type=int, help='Output col', required=True)
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--input', type=str, help='Input address pattern', required=True)
    ap_parser.add_argument('--output', type=str, help='Output address pattern', required=True)
    ap_parser.add_argument('--input_row', type=int, help='Input row', required=True)
    ap_parser.add_argument('--input_col', type=int, help='Input col', required=True)
    ap_parser.add_argument('--output_row', type=int, help='Output row', required=True)
    ap_parser.add_argument('--output_col', type=int, help='Output col', required=True)
    

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.in_mem, args.input_row, args.input_col, args.out_mem, args.output_row, args.output_col)
    elif args.subparser_name == 'ap':
        get_AP(args.input_row, args.input_col, args.output_row, args.output_col, args.input, args.output)

if __name__ == '__main__':
    main()



