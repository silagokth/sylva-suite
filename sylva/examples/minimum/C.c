#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
#include <cjson/cJSON.h>

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input_json_file> <output_json_file>\n", argv[0]);
        return 1;
    }

    int expected_minimum = INT_MAX;

    // -- Parse expected input JSON --
    FILE *f = fopen(argv[1], "r");
    if (!f) {
        perror("Error opening input JSON file");
        return 1;
    }

    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    rewind(f);

    char *data = malloc(len + 1);
    if (!data) {
        fprintf(stderr, "Memory allocation error\n");
        fclose(f);
        return 1;
    }
    fread(data, 1, len, f);
    data[len] = '\0';
    fclose(f);

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

    cJSON *item = NULL;
    cJSON_ArrayForEach(item, line) {
        cJSON *val_item = cJSON_GetObjectItem(item, "value");
        int value = (val_item && cJSON_IsNumber(val_item)) ? val_item->valueint : 0;
        if (value < expected_minimum) {
            expected_minimum = value;
        }
    }
    cJSON_Delete(json);

    // -- Parse output memory JSON --
    int output_minimum = INT_MIN;
    f = fopen(argv[2], "r");
    if (!f) {
        perror("Error opening memory JSON file");
        return 1;
    }

    fseek(f, 0, SEEK_END);
    len = ftell(f);
    rewind(f);

    data = malloc(len + 1);
    if (!data) {
        fprintf(stderr, "Memory allocation error\n");
        fclose(f);
        return 1;
    }
    fread(data, 1, len, f);
    data[len] = '\0';
    fclose(f);

    json = cJSON_Parse(data);
    free(data);

    if (!json) {
        fprintf(stderr, "JSON parse error: %s\n", cJSON_GetErrorPtr());
        return 1;
    }

    line = cJSON_GetObjectItem(json, "line");
    if (!cJSON_IsArray(line)) {
        fprintf(stderr, "Invalid JSON format: 'line' should be an array\n");
        cJSON_Delete(json);
        return 1;
    }

    cJSON_ArrayForEach(item, line) {
        cJSON *addr_item = cJSON_GetObjectItem(item, "address");
        cJSON *val_item = cJSON_GetObjectItem(item, "value");

        int address = (addr_item && cJSON_IsNumber(addr_item)) ? addr_item->valueint : 
                      (addr_item && cJSON_IsString(addr_item)) ? atoi(addr_item->valuestring) : 0;
        int value   = (val_item && cJSON_IsNumber(val_item)) ? val_item->valueint : 
                      (val_item && cJSON_IsString(val_item)) ? atoi(val_item->valuestring) : 0;

        if (address != 0) {
            fprintf(stderr, "Expected address to start at 0\n");
            cJSON_Delete(json);
            return 1;
        }

        output_minimum = value;
        break;  // Only consider the first entry
    }
    cJSON_Delete(json);

    if (output_minimum != expected_minimum) {
        fprintf(stderr, "Verification failed. Expected min %d, got %d\n", expected_minimum, output_minimum);
        return 1;
    }

    printf("Verified.\n");
    return 0;
}

