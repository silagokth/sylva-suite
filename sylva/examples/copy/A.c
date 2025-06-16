#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <output_file_path>\n", argv[0]);
        return 1;
    }
    const char *output_path = argv[1];
    
    FILE *f = fopen(output_path, "w");
    if (f == NULL) {
        perror("Error opening file");
        return 1;
    }

    fprintf(f, "{\n  \"line\": [\n");

    for (int i = 0; i < 16; i++) {
        fprintf(f, "    {\n      \"address\": %d,\n      \"value\": %d\n    }", i, i * 100);
        if (i != 15)
            fprintf(f, ",\n");
        else
            fprintf(f, "\n");
    }

    fprintf(f, "  ]\n}\n");
    fclose(f);
    printf("File created\n");
    return 0;
}

