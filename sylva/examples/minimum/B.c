#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cjson/cJSON.h>

typedef struct {
    char *global_image;
    char *in_mem;
    char *out_mem;
} Arguments;

void print_usage(const char *prog_name) {
    printf("Usage: %s [--global-image <path>] --in-mem <path> --out-mem <path>\n", prog_name);
}

int parse_arguments(int argc, char *argv[], Arguments *args) {
    if (argc < 5) { // minimum required args
        print_usage(argv[0]);
        return -1;
    }

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--global-image") == 0 && i + 1 < argc) {
            args->global_image = argv[++i];
        } else if (strcmp(argv[i], "--in-mem") == 0 && i + 1 < argc) {
            args->in_mem = argv[++i];
        } else if (strcmp(argv[i], "--out-mem") == 0 && i + 1 < argc) {
            args->out_mem = argv[++i];
        } else {
            printf("Unknown or incomplete argument: %s\n", argv[i]);
            print_usage(argv[0]);
            return -1;
        }
    }

    if (args->in_mem == NULL || args->out_mem == NULL) {
        printf("Error: --in-mem and --out-mem are required arguments.\n");
        print_usage(argv[0]);
        return -1;
    }

    return 0;
}


// Comparator for qsort
int compare_ints(const void *a, const void *b) {
    return (*(int *)a - *(int *)b);
}


/* Address space: 0x100 - 0x1FF */
const int base_address = 0x100;

int main(int argc, char *argv[]) {
    Arguments args = {0};

    if (parse_arguments(argc, argv, &args) != 0) {
        return 1;
    }
   
    // open and read the file
    FILE *fin = fopen(args.in_mem, "rb");
    if (fin == NULL) {
        perror("Error opening JSON file");
        return 1;
    }
 
    fseek(fin, 0, SEEK_END);
    long len = ftell(fin);
    rewind(fin);

    // Read file into buffer
    char *data = malloc(len + 1);
    if (!data) {
        fprintf(stderr, "Memory allocation error\n");
        fclose(fin);
        return 1;
    }
    fread(data, 1, len, fin);
    data[len] = '\0';
    fclose(fin);

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
    
    // Determine number of entries and collect values
    int max_entries = cJSON_GetArraySize(line);
    int *values = malloc(max_entries * sizeof(int));
    if (!values) {
        fprintf(stderr, "Memory allocation error\n");
        cJSON_Delete(json);
        return 1;
    
    }
    
    int i = 0;
    cJSON *item = NULL;
    cJSON_ArrayForEach(item, line) {
        cJSON *val = cJSON_GetObjectItem(item, "value");
        
        if (val && cJSON_IsNumber(val)) {
            values[i++] = val->valueint;
        } else if (val && cJSON_IsString(val)) {
            values[i++] = atoi(val->valuestring);
        } else {
            values[i++] = 0;  // Treat missing or invalid as 0
        }
    }

    // Sort the values
    qsort(values, max_entries, sizeof(int), compare_ints);
   
    // Create new JSON
    cJSON *new_root = cJSON_CreateObject();
    cJSON *new_line = cJSON_CreateArray();
    cJSON_AddItemToObject(new_root, "line", new_line);

    char str_num[12];
    for (int j = 0; j < max_entries; j++) {
        cJSON *entry = cJSON_CreateObject();
        snprintf(str_num, sizeof(str_num), "%d", j);
        cJSON_AddStringToObject(entry, "address", str_num);
        snprintf(str_num, sizeof(str_num), "%d", values[j]);
        cJSON_AddStringToObject(entry, "value", str_num);
        cJSON_AddItemToArray(new_line, entry);
    }

    // Write output JSON file
    FILE *fout = fopen(args.out_mem, "w");
    if (!fout) {
        fprintf(stderr, "Error opening output file\n");
        cJSON_Delete(new_root);
        free(values);
        return 1;
    }
    char *printed = cJSON_Print(new_root);
    fprintf(fout, "%s\n", printed);
    fclose(fout);

    free(printed);
    cJSON_Delete(new_root);
    cJSON_Delete(json);
    free(values);

    printf("Node B completes\n");
    return 0;
}

