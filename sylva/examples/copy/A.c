#include <stdio.h>
#include <stdlib.h>
#include <cjson/cJSON.h>

/* Address space: 0x000 - 0x0FF */
const int base_address = 0x000;

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <global_mem> <output_mem>\n", argv[0]);
        return 1;
    }

    // Open input file
    FILE *f = fopen(argv[1], "rb");
    if (!f) {
        perror("Error opening input file");
        return 1;
    }

    // Get file size
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    rewind(f);

    // Read contents
    char *data = malloc(len + 1);
    if (!data) {
        fprintf(stderr, "Memory allocation error\n");
        fclose(f);
        return 1;
    }
    fread(data, 1, len, f);
    data[len] = '\0';
    fclose(f);

    // Parse JSON
    cJSON *root = cJSON_Parse(data);
    free(data);
    if (!root) {
        fprintf(stderr, "JSON parse error: %s\n", cJSON_GetErrorPtr());
        return 1;
    }

    cJSON *line = cJSON_GetObjectItem(root, "line");
    if (!cJSON_IsArray(line)) {
        fprintf(stderr, "'line' is not an array in input JSON\n");
        cJSON_Delete(root);
        return 1;
    }

    // Create output JSON
    cJSON *out_root = cJSON_CreateObject();
    cJSON *out_line = cJSON_CreateArray();

    cJSON *item = NULL;
    cJSON_ArrayForEach(item, line) {
        cJSON *addr_item = cJSON_GetObjectItem(item, "address");
        cJSON *val_item = cJSON_GetObjectItem(item, "value");
        
        int address = (addr_item && cJSON_IsString(addr_item)) ? atoi(addr_item->valuestring) : 
                      (addr_item && cJSON_IsNumber(addr_item)) ? addr_item->valueint : 0;
        int value = (val_item && cJSON_IsString(val_item)) ? atoi(val_item->valuestring) :
                    (val_item && cJSON_IsNumber(val_item)) ? val_item->valueint : 0;
        address = base_address + address;
         
        if (address >= 0 && address <= 15) {
            cJSON *copy = cJSON_Duplicate(item, 1);
            cJSON_AddItemToArray(out_line, copy);
        }
    }

    cJSON_AddItemToObject(out_root, "line", out_line);

    // Write to output file
    FILE *out = fopen(argv[2], "w");
    if (!out) {
        perror("Error opening output file");
        cJSON_Delete(out_root);
        return 1;
    }

    char *out_data = cJSON_Print(out_root);
    fputs(out_data, out);
    fclose(out);

    // Cleanup
    free(out_data);
    cJSON_Delete(out_root);
    cJSON_Delete(root);

    printf("Node A completes\n");
    return 0;
}

