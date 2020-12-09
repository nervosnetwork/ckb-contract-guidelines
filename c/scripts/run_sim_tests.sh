#!/bin/bash
set -ex

ENVIRONMENT="$1"

SCRIPT_TOP="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
TOP="$SCRIPT_TOP/.."

for i in $(find $TOP/build/$ENVIRONMENT/dumped_tests -mindepth 1 -maxdepth 1 -type d); do
    bash $i/cmd
done
