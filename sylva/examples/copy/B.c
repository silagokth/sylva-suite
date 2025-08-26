#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cjson/cJSON.h>

typedef struct {
    char *global_image;
    char *in_mem;
    char *out_mem;
    int token;
} Arguments;

void print_usage(const char *prog_name) {
    printf("Usage: %s --global-image <path> [--in-mem <path>] --out-mem <path> --token <number-of-tokens>\n", prog_name);
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
        }  else if (strcmp(argv[i], "--token") == 0 && i + 1 < argc) {
            args->token = atoi(argv[++i]);
            if (args->token > 255) {
                printf("token size is more than 255");
                return -1;
            }
        } else {
            printf("Unknown or incomplete argument: %s\n", argv[i]);
            print_usage(argv[0]);
            return -1;
        }
    }

    if (args->in_mem == NULL || args->out_mem == NULL) {
        printf("Error: --in_mem and --out-mem are required arguments.\n");
        print_usage(argv[0]);
        return -1;
    }

    return 0;
}

/* Address space: 0x100 - 0x1FF */

int main(int argc, char *argv[]) {
    Arguments args = {0};

    if (parse_arguments(argc, argv, &args) != 0) {
        return 1;
    }

    const char *input_path = args.in_mem;
    const char *output_path = args.out_mem;

    FILE *fin = fopen(input_path, "rb");
    if (fin == NULL) {
        perror("Error opening input file");
        return 1;
    }
    FILE *fout = fopen(output_path, "wb");
    if (fout == NULL) {
        perror("Error opening output file");
        fclose(fin);
        return 1;
    }

    char buffer[4096];
    size_t bytes;

    while ((bytes = fread(buffer, 1, sizeof(buffer), fin)) > 0) {
        fwrite(buffer, 1, bytes, fout);
    }

    fclose(fin);
    fclose(fout);
    printf("Node B completes\n");
    return 0;
}

