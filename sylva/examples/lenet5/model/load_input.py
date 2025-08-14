#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import argparse
import util

def run(input_file, output_file, addr, size):
    memory = util.json2fpmem(input_file)
    output_memory = memory[addr:addr+size]
    util.fpmem2json(output_memory, output_file)

def get_AP(size, out_file):
    out_size = size
    util.gen_addr_pattern(out_size, out_file)

def main():
    parser = argparse.ArgumentParser(description='Exec')

    subparsers = parser.add_subparsers(dest="subparser_name")
    run_parser = subparsers.add_parser('run', help='Run function')
    run_parser.add_argument('--global-image', required=True, help="Path to global image file")
    run_parser.add_argument('--in-mem', required=False, help="Path to input memory JSON file")
    run_parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    run_parser.add_argument('--addr', type=int, help='starting_address', required=True)
    run_parser.add_argument('--size', type=int, help='Memory size', required=True)
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--output', type=str, help='Output address pattern', required=True)
    ap_parser.add_argument('--size', type=int, help='Memory size', required=True)

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.global_image, args.out_mem, args.addr, args.size)
    elif args.subparser_name == 'ap':
        get_AP(args.size, args.output)

if __name__ == '__main__':
    main()



