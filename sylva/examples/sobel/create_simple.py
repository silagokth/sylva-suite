import argparse
import json
import random
import math

def main():
    parser = argparse.ArgumentParser(description="Generate chunked and biased address-cycle JSON.")
    parser.add_argument("--filename", required=True, help="Output JSON filename")
    parser.add_argument("--size", type=int, required=True, help="Total number of addresses")
    parser.add_argument("--chunk-size", type=int, required=True, help="Number of addresses per chunk")
    parser.add_argument("--start", type=int, required=True, help="Start cycle value")

    args = parser.parse_args()

    size = args.size
    chunk_size = args.chunk_size
    start = args.start

    if chunk_size < 1:
        raise ValueError

    output = { "addr_ptrn": [] }

    count = 1
    cycle = start
    for i in range(size):
        output["addr_ptrn"].append({
            "address": str(i),
            "cycle": str(cycle)
        })
        if (count == chunk_size):
            count = 1
            cycle += 1 
        else:
            count += 1

    with open(args.filename, "w") as f:
        json.dump(output, f, indent=2)

if __name__ == "__main__":
    main()

