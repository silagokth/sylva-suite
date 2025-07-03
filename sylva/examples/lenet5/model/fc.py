#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import numpy as np
import json
import argparse
import sys
import random
import util

def mvm(m, v):
    """
    Matrix-vector multiplication

    Parameters:
    - m: 2D numpy array
    - v: 1D numpy array

    Returns:
    - result: 1D numpy array
    """
    return np.transpose(np.dot(m, np.transpose(v)))

def softmax(v):
    """
    Softmax function

    Parameters:
    - v: 1D numpy array

    Returns:
    - result: 1D numpy array
    """
    exp_v = np.exp(v)
    return exp_v / np.sum(exp_v)

def tanh(v):
    """
    Tanh function

    Parameters:
    - v: 1D numpy array

    Returns:
    - result: 1D numpy array
    """
    return np.tanh(v)

def fc(input_vector, weight_matrix, bias_vector, activation):
    """
    Fully connected layer

    Parameters:
    - input_vector: 1D numpy array
    - weight_matrix: 2D numpy array
    - bias_vector: 1D numpy array
    - activation: str, 'softmax' or 'tanh'

    Returns:
    - output_vector: 1D numpy array
    """
    output_vector = mvm(weight_matrix, input_vector) + bias_vector

    if activation == 'softmax':
        return softmax(output_vector)
    elif activation == 'tanh':
        return tanh(output_vector)
    else:
        return output_vector

def read_input(memory, input_size):
    storage_row_per_image_row = (input_size + 15) // 16
    current_row = 0
    input_image = []
    image_row = []
    for j in range(storage_row_per_image_row):
        image_row.extend(memory[current_row])
        current_row += 1
    input_image.extend(image_row[:input_size])
    return np.array(input_image)

def write_output(output_image, output_image_channel, output_image_size):
    storage_row_per_image_row = (output_image_size + 15) // 16
    current_row = 0
    memory = []
    for c in range(output_image_channel):
        for i in range(output_image_size):
            image_row = output_image[c][i].tolist()
            image_row.extend([0] * (storage_row_per_image_row * 16 - len(image_row)))
            for j in range(storage_row_per_image_row):
                memory.append(image_row[j*16:j*16+16])
                current_row += 1
    return memory

def matrix2mem(matrix, row, col):
    """
    Convert matrix to memory

    Parameters:
    - matrix: 2D numpy array
    - row: int
    - col: int

    Returns:
    - memory: list
    """
    storage_row_per_image_row = (col + 15) // 16
    memory = []
    for i in range(row):
        image_row = matrix[i].tolist()
        image_row.extend([0] * (storage_row_per_image_row * 16 - len(image_row)))
        for j in range(storage_row_per_image_row):
            memory.append(image_row[j*16:j*16+16])
    return memory

def mem2matrix(memory, row, col):
    """
    Convert memory to matrix

    Parameters:
    - memory: list
    - row: int
    - col: int

    Returns:
    - matrix: 2D numpy array
    """
    matrix = np.zeros((row, col))
    storage_row_per_image_row = (col + 15) // 16
    current_row = 0
    for i in range(row):
        matrix[i] = memory[current_row][:col]
        current_row += 1
    return matrix

def run(input_file, weight_file, bias_file, output_file, input_size, output_size, activation):
    memory = util.json2fpmem(input_file)
    input_image = util.mem2mat(np.array(memory), (1, input_size))
    memory = util.json2fpmem(weight_file)
    weight_matrix = util.mem2mat(np.array(memory), (output_size, input_size))
    memory = util.json2fpmem(bias_file)
    bias_vector = util.mem2mat(np.array(memory), (1, output_size))
    output_image = fc(input_image, weight_matrix, bias_vector, activation)
    out_mem = util.mat2mem(output_image)
    util.fpmem2json(out_mem, output_file)

def get_AP(input_size, output_size, in_file, out_file):
    input_storage_row_per_image_row = (input_size + 15) // 16
    output_storage_row_per_image_row = (output_size + 15) // 16
    in_size = input_storage_row_per_image_row
    out_size = output_storage_row_per_image_row
    util.gen_addr_pattern(in_size, in_file)
    util.gen_addr_pattern(out_size, out_file)

def main():
    parser = argparse.ArgumentParser(description='Exec')

    subparsers = parser.add_subparsers(dest="subparser_name")
    run_parser = subparsers.add_parser('run', help='Run function')
    run_parser.add_argument('--global-image', required=False, help="Path to global image file")
    run_parser.add_argument('--in-mem', required=True, help="Path to input memory JSON file")
    run_parser.add_argument('--out-mem', required=True, help="Path to output memory JSON file")
    run_parser.add_argument('--input_size', type=int, help='Input size', required=True)
    run_parser.add_argument('--output_size', type=int, help='output size', required=True)
    run_parser.add_argument('--activation', type=str, help='Activation', default='tanh')
    run_parser.add_argument('--weight', type=str, help='Weight file', required=True)
    run_parser.add_argument('--bias', type=str, help='Bias file', required=True)
    ap_parser = subparsers.add_parser('ap', help='Get address pattern')
    ap_parser.add_argument('--input', type=str, help='Input address pattern', required=True)
    ap_parser.add_argument('--output', type=str, help='Output address pattern', required=True)
    ap_parser.add_argument('--input_size', type=int, help='Input size', required=True)
    ap_parser.add_argument('--output_size', type=int, help='output size', required=True)

    args = parser.parse_args()
    if args.subparser_name == 'run':
        run(args.in_mem, args.weight, args.bias, args.out_mem, args.input_size, args.output_size, args.activation)
    elif args.subparser_name == 'ap':
        get_AP(args.input_size, args.output_size, args.input, args.output)

if __name__ == '__main__':
    main()



