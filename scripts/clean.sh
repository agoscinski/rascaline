#!/usr/bin/env bash
set -ux
ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"

rm -rf $ROOT/build
rm -rf $ROOT/dist
rm -rf $ROOT/.tox

rm -rf $ROOT/python/*.egg-info
rm -rf $ROOT/python/*/__pycache__

rm -rf $ROOT/tests/c_api/build
