#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import json
import argparse
import sys
import random
import util

def run(input_file, output_file, addr, size):
    original_memory = util.json2fpmem(output_file)
    memory = util.json2fpmem(input_file)
    total_size = max(addr + size, len(original_memory))
    final_memory = []
    for i in range(total_size):
        if i < addr:
            final_memory.append(original_memory[i])
        elif i < addr + size:
            final_memory.append(memory[i - addr])
        else:
            final_memory.append(original_memory[i])
    final_memory = np.array(final_memory)
    util.fpmem2json(final_memory, output_file)

def get_AP(size, in_file):
    in_size = size
    util.gen_addr_pattern(in_size, in_file)

def main():
    parser = argparse.ArgumentParser(description='Exec')

    subparsers = parser.add_subparsers(dest="subparser_name")
    run_parser = subparsers.add_parser('run', help='Run function')
    run_parser.add_argument('--global-image', required=True, help="Path to global image file")
    run_parser.add_argument('--in-mem', required=True, help="Path to input memory JSON file")
    run_parser.add_argument('--out-mem', required=False, help="Path to output memory JSON file")
    run_parser.add_argument('--addr', type=int, help='starting_address', required=True)
    run_parser.add_argument('--size', type=int, help='vector size', required=True)
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--input', type=str, help='Input address pattern', required=True)
    ap_parser.add_argument('--size', type=int, help='vector size', required=True)

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.in_mem, args.global_image, args.addr, args.size)
    elif args.subparser_name == 'ap':
        get_AP(args.size, args.input)

if __name__ == '__main__':
    main()



