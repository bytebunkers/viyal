#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

struct Math {
    int64_t (*add)(struct Math* this, int64_t a, int64_t b);
};

int64_t Math_add_impl(struct Math* this, int64_t a, int64_t b) {
    return a + b;
}

struct Math* Math_new() {
    struct Math* obj = calloc(1, sizeof(struct Math));
    obj->add = Math_add_impl;
    return obj;
}

int main() {
    struct Math* m = Math_new();
    int64_t result = (m)->add(m, 10, 5);
    printf("%ld\n", result);
    return 0;
}
