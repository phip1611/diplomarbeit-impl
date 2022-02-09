#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

const uint32_t DIM = 3;
int main() {
    // Two 3*x matrices on the Heap with field values: 0, 1, 2, 3, ...
    uint32_t * matrix1 = malloc(sizeof(uint32_t) * DIM * DIM);
    uint32_t * matrix2 = malloc(sizeof(uint32_t) * DIM * DIM);
    for (uint32_t i = 0; i < DIM; i++) {
        for (uint32_t j = 0; j < DIM; j++) {
            uint32_t num = i * DIM + j;
            matrix1[num] = num;
            matrix2[num] = num;
        }
    }

    // Target matrix on Heap; matrix multiplication
    uint32_t * res_matrix = malloc(sizeof(uint32_t) * DIM * DIM);
    for (int i = 0; i < DIM; i++) {
        for (int j = 0; j < DIM; j++) {
            uint32_t sum = 0;
            for (int k = 0; k < DIM; k++) {
                sum += matrix1[i * DIM + k] * matrix2[k * DIM + j];
            }
            res_matrix[i * DIM + j] = sum;
        }
    }

    // print
    printf("[\n");
    for (int i = 0; i < DIM; i++) {
        printf("  [");
        for (int j = 0; j < DIM; j++) {
            printf("%u,", res_matrix[i * DIM + j]);
        }
        printf("]\n");
    }
    printf("]\n");
    free(matrix1);
    free(matrix2);
    free(res_matrix);
}
