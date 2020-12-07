#include "blake2b.h"
#include "ckb_dlfcn.h"

#define RISCV_PGSIZE 4096

int main() {
  uint8_t data[33];
  uint64_t len = 33;
  int ret = ckb_load_cell_data(data, &len, 0, 0, CKB_SOURCE_GROUP_OUTPUT);
  if (ret != CKB_SUCCESS) {
    return ret;
  }
  if (len < 33) {
    return -10;
  }

  uint8_t code_buffer[128 * 1024] __attribute__((aligned(RISCV_PGSIZE)));
  uint64_t consumed_size = 0;
  void *handle = NULL;
  ret = ckb_dlopen2(data, data[32], code_buffer, 128 * 1024, &handle,
                    &consumed_size);
  if (ret != CKB_SUCCESS) {
    return ret;
  }
  int (*validate_func)(size_t);
  *(void **)(&validate_func) = ckb_dlsym(handle, "validate_tx");
  if (ret != CKB_SUCCESS) {
    return -11;
  }
  return validate_func(2);
}
