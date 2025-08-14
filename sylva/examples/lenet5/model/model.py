#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import sys
import os
import random
import util
import numpy as np

def main():

    ret = os.system('mkdir -p work')
    if ret != 0:
        sys.exit(1)
    ret = os.system('mkdir -p work/ref/mem')
    if ret != 0:
        sys.exit(1)
    ret = os.system('mkdir -p work/sim/mem')
    if ret != 0:
        sys.exit(1)
    ret = os.system('mkdir -p work/sim/data')
    if ret != 0:
        sys.exit(1)
    

    # generate global memory
    image_size = 32
    image_channel = 1
    image = np.array([[(random.random()-0.5)*0.5 for _ in range(image_size)] for _ in range(image_channel*image_size)])
    memory = util.mat2mem(image)
    util.fpmem2json(memory, 'global_mem.json')
    
    # conv1, conv2, conv3 kernel
    kernel_size = 5
    kernel_channel = [6, 16, 120]
    output_size = [28, 10, 1]
    for i in range(len(kernel_channel)):
        kernel = np.array([[(random.random()-0.5)*0.5 for _ in range(kernel_size)] for _ in range(kernel_channel[i]*kernel_size)])
        memory = util.mat2mem(kernel)
        util.fpmem2json(memory, 'conv{}_kernel.json'.format(i+1))
        bias = np.array([[(random.random()-0.5)*0.5 for _ in range(output_size[i])] for _ in range(kernel_channel[i]*output_size[i])])
        memory = util.mat2mem(bias)
        util.fpmem2json(memory, 'conv{}_bias.json'.format(i+1))
    
    # fc1, fc2 weight and bias
    fc_size = [120, 84, 10]
    for i in range(len(fc_size)-1):
        input_size = fc_size[i]
        output_size = fc_size[i+1]
        weight = np.array([[(random.random()-0.5)*0.5 for _ in range(input_size)] for _ in range(output_size)])
        memory = util.mat2mem(weight)
        util.fpmem2json(memory, 'fc{}_weight.json'.format(i+1))
        bias = np.array([(random.random()-0.5)*0.5 for _ in range(output_size)])
        memory = util.mat2mem(bias)
        util.fpmem2json(memory, 'fc{}_bias.json'.format(i+1))
    
    ret = os.system('cp global_mem.json work/sim/mem/global_mem.json')
    if ret != 0:
        sys.exit(1)

    # load input
    ret = os.system('python load_input.py run --input global_mem.json --output load_input_outMem.json --addr 0 --size 64')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python load_input.py ap --size 64 --output load_input_outAP.json')
    if ret != 0:
        sys.exit(1)

    # conv1
    ret = os.system('cp load_input_outMem.json conv1_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py run --input conv1_inMem.json --output conv1_outMem.json --input_image_channel 1 --input_image_size 32 --kernel_channel 6 --kernel_size 5 --stride 1 --kernel conv1_kernel.json --bias conv1_bias.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py ap --input_image_channel 1 --input_image_size 32 --kernel_channel 6 --kernel_size 5 --stride 1 --input conv1_inAP.json --output conv1_outAP.json')
    if ret != 0:
        sys.exit(1)

    # pooling1
    ret = os.system('cp conv1_outMem.json pooling1_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python pooling.py run --input pooling1_inMem.json --output pooling1_outMem.json --input_image_channel 6 --input_image_size 28 --kernel_size 2 --stride 2 --mode average')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python pooling.py ap --input_image_channel 6 --input_image_size 28 --kernel_size 2 --stride 2 --input pooling1_inAP.json --output pooling1_outAP.json')
    if ret != 0:
        sys.exit(1)

    # conv2
    ret = os.system('cp pooling1_outMem.json conv2_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py run --input conv2_inMem.json --output conv2_outMem.json --input_image_channel 6 --input_image_size 14 --kernel_channel 16 --kernel_size 5 --stride 1 --kernel conv2_kernel.json --bias conv2_bias.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py ap --input_image_channel 6 --input_image_size 14 --kernel_channel 16 --kernel_size 5 --stride 1 --input conv2_inAP.json --output conv2_outAP.json')
    if ret != 0:
        sys.exit(1)

    # pooling2
    ret = os.system('cp conv2_outMem.json pooling2_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python pooling.py run --input pooling2_inMem.json --output pooling2_outMem.json --input_image_channel 16 --input_image_size 10 --kernel_size 2 --stride 2 --mode average')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python pooling.py ap --input_image_channel 16 --input_image_size 10 --kernel_size 2 --stride 2 --input pooling2_inAP.json --output pooling2_outAP.json')
    if ret != 0:
        sys.exit(1)

    # conv3
    ret = os.system('cp pooling2_outMem.json conv3_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py run --input conv3_inMem.json --output conv3_outMem.json --input_image_channel 16 --input_image_size 5 --kernel_channel 120 --kernel_size 5 --stride 1 --kernel conv3_kernel.json --bias conv3_bias.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python conv.py ap --input_image_channel 16 --input_image_size 5 --kernel_channel 120 --kernel_size 5 --stride 1 --input conv3_inAP.json --output conv3_outAP.json')
    if ret != 0:
        sys.exit(1)
    
    # reshape
    ret = os.system('cp conv3_outMem.json reshape_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python reshape.py run --input reshape_inMem.json --input_row 120 --input_col 1 --output reshape_outMem.json --output_row 1 --output_col 120')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python reshape.py ap --input_row 120 --input_col 1 --output_row 1 --output_col 120 --input reshape_inAP.json --output reshape_outAP.json')
    if ret != 0:
        sys.exit(1)
    
    # fc1
    ret = os.system('cp reshape_outMem.json fc1_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python fc.py run --input fc1_inMem.json --output fc1_outMem.json --input_size 120 --output_size 84 --weight fc1_weight.json --bias fc1_bias.json --activation=tanh')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python fc.py ap --input_size 120 --output_size 84 --input fc1_inAP.json --output fc1_outAP.json')
    if ret != 0:
        sys.exit(1)
    
    # fc2
    ret = os.system('cp fc1_outMem.json fc2_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python fc.py run --input fc2_inMem.json --output fc2_outMem.json --input_size 84 --output_size 10 --weight fc2_weight.json --bias fc2_bias.json --activation=softmax')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python fc.py ap --input_size 84 --output_size 10 --input fc2_inAP.json --output fc2_outAP.json')
    if ret != 0:
        sys.exit(1)
    
    # store output
    ret = os.system('cp fc2_outMem.json store_output_inMem.json')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python store_output.py run --input store_output_inMem.json --output global_mem.json --addr 64 --size 1')
    if ret != 0:
        sys.exit(1)
    ret = os.system('python store_output.py ap --size 1 --input store_output_inAP.json')
    if ret != 0:
        sys.exit(1)
    
    # copy files
    ret = os.system('cp *AP.json work/sim')
    if ret != 0:
        sys.exit(1)
    ret = os.system('cp *weight.json work/sim/data')
    if ret != 0:
        sys.exit(1)
    ret = os.system('cp *bias.json work/sim/data')
    if ret != 0:
        sys.exit(1)
    ret = os.system('cp *kernel.json work/sim/data')
    if ret != 0:
        sys.exit(1)
    ret = os.system('cp global_mem.json work/ref/mem')
    if ret != 0:
        sys.exit(1)
    
    # remove all json files
    ret = os.system('rm *.json')
    if ret != 0:
        sys.exit(1)

    
if __name__ == '__main__':
    main()
