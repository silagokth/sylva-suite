#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cjson/cJSON.h>

void load_json(const char *filename, cJSON **root) {
    FILE *f = fopen(filename, "rb");
    if (!f) {
        perror("Opening file");
        exit(1);
    }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    rewind(f);

    char *data = malloc(len + 1);
    if (!data) {
        fprintf(stderr, "Memory allocation failed\n");
        fclose(f);
        exit(1);
    }

    fread(data, 1, len, f);
    data[len] = '\0';
    fclose(f);

    *root = cJSON_Parse(data);
    free(data);

    if (!*root) {
        fprintf(stderr, "JSON parse error: %s\n", cJSON_GetErrorPtr());
        exit(1);
    }
}

/* Address space: 0x200 - 0x2FF */
const int base_address = 0x200;

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <global_mem> <input_mem>\n", argv[0]);
        return 1;
    }

    cJSON *input_mem = NULL;
    cJSON *global_mem = NULL;
    load_json(argv[1], &global_mem);
    load_json(argv[2], &input_mem);

    cJSON *input_line = cJSON_GetObjectItem(input_mem, "line");
    cJSON *global_line = cJSON_GetObjectItem(global_mem, "line");

    if (!cJSON_IsArray(input_line) || !cJSON_IsArray(global_line)) {
        fprintf(stderr, "Invalid format: 'line' should be an array in both files\n");
        cJSON_Delete(input_mem);
        cJSON_Delete(global_mem);
        return 1;
    }

    // Step 1: Store only the first value 
    int result = -1;
    cJSON *item = NULL;
    cJSON_ArrayForEach(item, input_line) {
        cJSON *addr_item = cJSON_GetObjectItem(item, "address");
        cJSON *val_item = cJSON_GetObjectItem(item, "value");
        
        int address = (addr_item && cJSON_IsString(addr_item)) ? atoi(addr_item->valuestring) : 
                      (addr_item && cJSON_IsNumber(addr_item)) ? addr_item->valueint : 0;
        int value = (val_item && cJSON_IsString(val_item)) ? atoi(val_item->valuestring) :
                    (val_item && cJSON_IsNumber(val_item)) ? val_item->valueint : 0;

        if (address == 0) {
            result = value;
        }
    }

    // Step 2: Update global line
    char str_num[12];
    cJSON_ArrayForEach(item, global_line) {
        cJSON *addr_item = cJSON_GetObjectItem(item, "address");
        int address = (addr_item && cJSON_IsString(addr_item)) ? atoi(addr_item->valuestring) : 
                      (addr_item && cJSON_IsNumber(addr_item)) ? addr_item->valueint : 0;
        
        if (address == base_address + 0) {
            snprintf(str_num, sizeof(str_num), "%d", result);
            cJSON_ReplaceItemInObject(item, "value", cJSON_CreateString(str_num));
        }
    }

    // Step 3: Write to output file
    FILE *out = fopen(argv[1], "w");
    if (!out) {
        perror("Writing output");
        cJSON_Delete(input_mem);
        cJSON_Delete(global_mem);
        return 1;
    }

    char *out_data = cJSON_Print(global_mem);
    fputs(out_data, out);
    fclose(out);

    free(out_data);
    cJSON_Delete(input_mem);
    cJSON_Delete(global_mem);

    printf("Node C completes\n");
    return 0;
}

