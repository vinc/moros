#!/bin/sh

set -e

# TODO: Add steps to create image with VBoxManage createvm
make image output=video keyboard=dvorak nic=pcnet
qemu-img convert -f raw -O vdi disk.img disk.vdi -o size=32M
VBoxManage internalcommands sethduuid disk.vdi dbbfad68-c3d1-4c9a-828f-7e4db4e9488e
VBoxManage startvm Moros
