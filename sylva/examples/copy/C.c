#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cjson/cJSON.h>

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <input_json_file>\n", argv[0]);
        return 1;
    }

    const char *filename = argv[1];
    FILE *f = fopen(filename, "rb");
    if (!f) {
        perror("Error opening JSON file");
        return 1;
    }

    // Get file size
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    rewind(f);

    // Read file into buffer
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
    cJSON *json = cJSON_Parse(data);
    free(data);

    if (!json) {
        fprintf(stderr, "JSON parse error: %s\n", cJSON_GetErrorPtr());
        return 1;
    }

    cJSON *line = cJSON_GetObjectItem(json, "line");
    if (!cJSON_IsArray(line)) {
        fprintf(stderr, "Invalid JSON format: 'line' should be an array\n");
        cJSON_Delete(json);
        return 1;
    }

    int failed = 0;
    int count = 0;
    cJSON *item = NULL;
    cJSON_ArrayForEach(item, line) {
        cJSON *addr_item = cJSON_GetObjectItem(item, "address");
        cJSON *val_item = cJSON_GetObjectItem(item, "value");
        
        // considering a missing field as zero value 
        int address = (addr_item) ? atoi(addr_item->valuestring) : 0;
        int value = (val_item) ? atoi(val_item->valuestring) : 0;
            
        if (value != count * 100 || address != count) {
            fprintf(stderr, "Mismatch: address=%d, value=%d (expected %d)\n",
                    address, value, count * 100);
            failed = 1;
        }
        count++;
    }
    cJSON_Delete(json);

    if (failed) {
        fprintf(stderr, "Verification failed.\n");
        return 1;
    }

    printf("Verified.\n");
    return 0;
}

