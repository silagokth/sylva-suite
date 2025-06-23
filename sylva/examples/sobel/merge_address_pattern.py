import argparse
import json

def read_json(filename):
    with open(filename, "r") as f:
        return json.load(f)

def write_json(data, filename):
    with open(filename, "w") as f:
        json.dump(data, f, indent=2)

def get_max_address(json_data):
    return max(int(entry["address"]) for entry in json_data["addr_ptrn"])

def shift_addresses(json_data, offset):
    shifted = []
    for entry in json_data["addr_ptrn"]:
        new_entry = {
            "address": str(int(entry["address"]) + offset),
            "cycle": entry["cycle"]
        }
        shifted.append(new_entry)
    return shifted

def main():
    parser = argparse.ArgumentParser(description="Merge two JSON files with address shift.")
    parser.add_argument("--file1", required=True, help="First input JSON file")
    parser.add_argument("--file2", required=True, help="Second input JSON file to be shifted")
    parser.add_argument("--output", required=True, help="Output merged JSON file")
    args = parser.parse_args()

    # Read both files
    json1 = read_json(args.file1)
    json2 = read_json(args.file2)

    # Determine max address in first file
    max_address = get_max_address(json1)

    # Shift all addresses in second file
    shifted_entries = shift_addresses(json2, max_address + 1)

    # Merge both
    merged = {
        "addr_ptrn": json1["addr_ptrn"] + shifted_entries
    }

    # Write to output
    write_json(merged, args.output)
    print(f"Combined file written to: {args.output}")

if __name__ == "__main__":
    main()

