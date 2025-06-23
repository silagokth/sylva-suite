import argparse
import json
import random
import math

def biased_random(low, high, bias_towards_high=True):
    """Return a biased random integer between low and high."""
    bias = random.random()
    if not bias_towards_high:
        bias = 1 - bias  # Invert for biasing toward low
    return int(low + (high - low + 1) * (bias ** 2))  # quadratic bias

def main():
    parser = argparse.ArgumentParser(description="Generate chunked and biased address-cycle JSON.")
    parser.add_argument("--filename", required=True, help="Output JSON filename")
    parser.add_argument("--size", type=int, required=True, help="Total number of addresses")
    parser.add_argument("--chunk-size", type=int, required=True, help="Number of addresses per chunk")
    parser.add_argument("--low-limit", type=int, required=True, help="Minimum cycle value")
    parser.add_argument("--high-limit", type=int, required=True, help="Maximum cycle value")

    args = parser.parse_args()

    size = args.size
    chunk_size = args.chunk_size
    low = args.low_limit
    high = args.high_limit

    if size < chunk_size:
        raise ValueError("Size must be greater than or equal to chunk size.")

    num_chunks = math.ceil(size / chunk_size)
    cycle_range = high - low + 1
    cycle_chunk_size = math.ceil(cycle_range / num_chunks)

    output = { "addr_ptrn": [] }

    for chunk in range(num_chunks):
        start_addr = chunk * chunk_size
        end_addr = min((chunk + 1) * chunk_size, size)

        cycle_start = low + chunk * cycle_chunk_size
        cycle_end = min(low + (chunk + 1) * cycle_chunk_size - 1, high)

        for addr in range(start_addr, end_addr):
            cycle = biased_random(cycle_start, cycle_end, bias_towards_high=True)
            output["addr_ptrn"].append({
                "address": str(addr),
                "cycle": str(cycle)
            })

    with open(args.filename, "w") as f:
        json.dump(output, f, indent=2)

if __name__ == "__main__":
    main()

