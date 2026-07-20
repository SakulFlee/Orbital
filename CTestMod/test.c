#include <stdint.h>

__attribute__((export_name("test"))) uint64_t test(uint8_t *ptr, uint32_t len) {
  float *val = (float *)ptr;

  *val += 123.0f;

  uint64_t out_ptr = (uint64_t)(uintptr_t)ptr;
  uint64_t out_len = (uint64_t)len;

  return (out_ptr << 32) | out_len;
}
