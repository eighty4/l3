#!/bin/bash
set -e

FILE=$1
VERSION=$2

awk -v p="## $VERSION" '$0 ~ p{f=1; next} /^## .*$/{f=0} f' "$FILE" | grep .
