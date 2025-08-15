import json
import sys

if len(sys.argv) != 4:
    print(f"Usage: {sys.argv[0]} <file1.json> <file2.json> <output.json>")
    sys.exit(1)

file1 = sys.argv[1]
file2 = sys.argv[2]
output_file = sys.argv[3]

# Load the two JSON files
with open(file1, "r") as f:
    data1 = json.load(f)

with open(file2, "r") as f:
    data2 = json.load(f)

# Offset to add to second file's addresses
offset = 12800

# Update addresses in second file
for entry in data2["line"]:
    entry["address"] = str(int(entry["address"]) + offset)

# Combine the two lists
combined_data = {
    "line": data1["line"] + data2["line"]
}

# Save the result
with open(output_file, "w") as f:
    json.dump(combined_data, f, indent=2)

print(f"Combined JSON written to {output_file}")

