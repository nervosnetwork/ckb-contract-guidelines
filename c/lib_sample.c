#define __SHARED_LIBRARY__ 1
#include "ckb_type_id.h"

__attribute__((visibility("default"))) int validate_tx(size_t offset) {
  uint8_t type_id[32];
  int ret = ckb_load_type_id_from_script_args(offset, type_id);
  if (ret != CKB_SUCCESS) {
    return ret;
  }
  return ckb_validate_type_id(type_id);
}
