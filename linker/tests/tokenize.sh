#!/bin/bash

INDIR=$1
OUTDIR=$2

# Loop 100 times
for i in {1..20}; do
    # Run tokenize program and save output to input file
  /home/frankeh/Public/lab1/tokenizer ${INDIR}/input-$i > ${OUTDIR}/out-$i
done