#!/bin/bash
set -ex

ENVIRONMENT="$1"

SCRIPT_TOP="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
TOP="$SCRIPT_TOP/.."

for i in $(find $TOP/build/$ENVIRONMENT/dumped_tests -mindepth 1 -maxdepth 1 -type d); do
    if [[ "$i" =~ ^.*failure$ ]]; then
        CKB_TX_FILE=$i/tx.json CKB_RUNNING_SETUP=$i/setup.json build/$ENVIRONMENT/simple_udt_sim || error_code=$?
        if [[ "$error_code" -eq 0 ]]; then
           echo "Failure test passes!"
           exit 1
        fi
    else
        CKB_TX_FILE=$i/tx.json CKB_RUNNING_SETUP=$i/setup.json build/$ENVIRONMENT/simple_udt_sim
    fi
done
