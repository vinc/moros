#!/bin/sh

img="disk.img"
path="/tmp/moros"

# pip install fusepy
mkdir -p $path
echo "Mounting $img in $path"
python run/moros-fuse.py $img $path
