#include <stdio.h>
#include <stdlib.h>

/* Address space: 0x100 - 0x1FF */

int main(int argc, char *argv[]) {
    if (argc != 4) {
        fprintf(stderr, "Usage: %s <global_mem> <input_mem> <output_mem>\n", argv[0]);
        return 1;
    }

    const char *input_path = argv[2];
    const char *output_path = argv[3];

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
    printf("Node B completes\n", input_path, output_path);
    return 0;
}

