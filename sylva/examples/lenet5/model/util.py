import numpy as np
import json
import random
from math import prod

def hex_string_to_int_vector(hex_string):
    # check hex string length to be 64
    if len(hex_string) != 64:
        print("Error: hex string length is not 64")
        sys.exit(1)
    vector = []
    for i in range(0, 64, 4):
        hex_str_num = hex_string[i:i+4]
        # convert to integer, considering also negative numbers
        int_num = int(hex_str_num, 16)
        if int_num > 32767:
            int_num = int_num - 65536
        vector.append(int_num)
    return vector

def int_vector_to_hex_string(int_vector):
    if len(int_vector) != 16:
        print("Error: int vector length is not 16")
        sys.exit(1)
    for i in range(len(int_vector)):
        if int_vector[i] < 0:
            int_vector[i] = int_vector[i] + 65536
    hex_string = ''.join([format(x, '04x') for x in int_vector])
    return hex_string

def float_vector_to_hex_string(float_vector):
    if len(float_vector) != 16:
        print("Error: float vector length is not 16")
        sys.exit(1)
    int_vector = [int(x*100) for x in float_vector]
    hex_string = int_vector_to_hex_string(int_vector)
    return hex_string

def hex_string_to_float_vector(hex_string):
    if len(hex_string) != 64:
        print("Error: hex string length is not 64")
        sys.exit(1)
    int_vector = hex_string_to_int_vector(hex_string)
    float_vector = [float(x/100.0) for x in int_vector]
    return float_vector

def mat2mem(matrix):
    """
    Convert matrix to memory

    Parameters:
    - matrix: 2D numpy array

    Returns:
    - memory: np.array
    """
    if len(matrix.shape) < 1:
        print("Error: matrix shape length is less than 1")
        sys.exit(1)
    elif len(matrix.shape) == 1:
        row = 1
        col = matrix.shape[0]
        new_matrix = matrix.reshape(row, col)
    else:
        row = prod(matrix.shape[0:-1])
        col = matrix.shape[-1]
        new_matrix = matrix.reshape(row, col)

    row, col = new_matrix.shape
    storage_row_per_image_row = (col + 15) // 16
    memory = []
    for i in range(row):
        image_row = new_matrix[i].tolist()
        image_row.extend([0] * (storage_row_per_image_row * 16 - len(image_row)))
        for j in range(storage_row_per_image_row):
            memory.append(image_row[j*16:j*16+16])
    return np.array(memory)

def mem2mat(memory, shape):
    """
    Convert memory to matrix

    Parameters:
    - memory: np.array
    - shape: tuple

    Returns:
    - matrix: 2D numpy array
    """
    # check memory shape has to be 2d and its col has to be 16
    assert len(memory.shape) == 2
    assert memory.shape[-1] == 16

    if len(shape) < 1:
        print("Error: shape length is less than 1")
        sys.exit(1)
    elif len(shape) == 1:
        col = shape[0]
        row = 1
    else:
        col = shape[-1]
        row = prod(shape[0:-1])
    matrix = np.zeros((row, col))
    storage_row_per_image_row = (col + 15) // 16

    # check memory length has to be equal to row * storage_row_per_image_row
    assert len(memory) == row * storage_row_per_image_row

    # reshape memory according to shape
    new_memory = np.array(memory).reshape(row, storage_row_per_image_row, 16)
    for i in range(row):
        row_data = [0 for _ in range(col)]
        for j in range(storage_row_per_image_row):
            if j == storage_row_per_image_row - 1:
                row_data[j*16:j*16+16] = new_memory[i][j][:col]
            else:
                row_data[j*16:j*16+16] = new_memory[i][j]
        matrix[i] = row_data[:col]
    new_matrix = matrix.reshape(shape)
    return new_matrix

def json2intmem(json_file):
    with open(json_file, 'r') as f:
        data = json.load(f)
    memory = []
    for line in data['line']:
        memory.append(hex_string_to_int_vector(line['value']))
    return np.array(memory)

def intmem2json(memory, json_file):
    data = {}
    data['line'] = []
    for i in range(len(memory)):
        hex_string = int_vector_to_hex_string(memory[i])
        data['line'].append({"address": str(i), "value": hex_string})
    with open(json_file, 'w') as f:
        json.dump(data, f)

def json2fpmem(json_file):
    with open(json_file, 'r') as f:
        data = json.load(f)
    memory = []
    for line in data['line']:
        memory.append(hex_string_to_float_vector(line['value']))
    return np.array(memory)

def fpmem2json(memory, json_file):
    data = {}
    data['line'] = []
    for i in range(len(memory)):
        hex_string = float_vector_to_hex_string(memory[i])
        data['line'].append({"address": str(i), "value": hex_string})
    with open(json_file, 'w') as f:
        json.dump(data, f)

def gen_addr_pattern(size, out_file, delta=5):
    addr_pattern = []
    curr_position = 0
    for i in range(size):
        curr_position = max(0, random.randint(curr_position - delta//2, curr_position + delta*2))
        addr_pattern.append({"address": str(i), "cycle": str(curr_position)})
    with open(out_file, 'w+') as f:
        json.dump({"addr_ptrn": addr_pattern}, f)

def test_mat2mem():
    matrix = np.array([[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        [17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]])
    memory = mat2mem(matrix)
    expected_memory = np.array([[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                       [17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]])
    assert np.array_equal(memory, expected_memory)

    matrix = np.array([[[1, 2, 3, 4, 5, 6, 7, 8],[9, 10, 11, 12, 13, 14, 15, 16]],
                        [[17, 18, 19, 20, 21, 22, 23, 24],[25, 26, 27, 28, 29, 30, 31, 32]]])
    memory = mat2mem(matrix)
    expected_memory = np.array([[1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
                          [9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0, 0, 0, 0, 0],
                          [17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 0],
                          [25, 26, 27, 28, 29, 30, 31, 32, 0, 0, 0, 0, 0, 0, 0, 0]])
    assert np.array_equal(memory, expected_memory)

def test_mem2mat():
    memory = np.array([[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              [17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]])
    shape = (2, 8)
    matrix = mem2mat(memory, shape)
    expected_matrix = np.array([[1, 2, 3, 4, 5, 6, 7, 8],
                                [17, 18, 19, 20, 21, 22, 23, 24]])
    assert np.array_equal(matrix, expected_matrix)

    memory = np.array([[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              [17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 0],
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              [17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 0],
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              [17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 0],
              [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              [17, 18, 19, 20, 21, 22, 23, 24, 0, 0, 0, 0, 0, 0, 0, 0]])
    shape = (2, 2, 24)
    matrix = mem2mat(memory, shape)
    expected_matrix = np.array([[[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]],
                                [[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]]])
    assert np.array_equal(matrix, expected_matrix)

if __name__ == "__main__":
    test_mat2mem()
    test_mem2mat()
    print("All tests pass")