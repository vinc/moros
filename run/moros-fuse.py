#!/usr/bin/env python

import logging
import os
from errno import ENOENT
from fuse import FUSE, FuseOSError, Operations, LoggingMixIn
from stat import S_IFDIR, S_IFREG

BLOCK_SIZE = 512
SUPERBLOCK_ADDR = 4096 * BLOCK_SIZE
BITMAP_ADDR = SUPERBLOCK_ADDR + 2 * BLOCK_SIZE

class MorosFuse(LoggingMixIn, Operations):
    chmod = None
    chown = None
    create = None
    mkdir = None
    readlink = None
    rename = None
    rmdir = None
    symlink = None
    truncate = None
    unlink = None
    utimens = None
    write = None

    def __init__(self, path):
        self.block_size = BLOCK_SIZE
        self.image = open(path, "rb")
        self.image.seek(SUPERBLOCK_ADDR)
        superblock = self.image.read(self.block_size)
        assert superblock[0:8] == b"MOROS FS" # Signature
        assert superblock[8] == 1 # Version
        assert self.block_size == 2 << (8 + superblock[9])
        self.block_count = int.from_bytes(superblock[10:14], "big")
        self.alloc_count = int.from_bytes(superblock[14:18], "big")

        bs = 8 * self.block_size # number of bits per bitmap block
        total = self.block_count
        rest = (total - (BITMAP_ADDR // self.block_size)) * bs // (bs + 1)
        self.data_addr = BITMAP_ADDR + (rest // bs) * self.block_size

    def destroy(self, path):
        self.image.close()
        return

    def getattr(self, path, fh=None):
        (kind, addr, size, time, name) = self.__scan(path)
        if addr == 0:
            raise FuseOSError(ENOENT)
        mode = S_IFDIR | 0o755 if kind == 0 else S_IFREG | 0o644
        return { "st_atime": 0, "st_mtime": time, "st_uid": 0, "st_gid": 0, "st_mode": mode, "st_size": size }

    def read(self, path, size, offset, fh):
        (kind, next_block_addr, size, time, name) = self.__scan(path)
        res = b""
        while next_block_addr != 0:
            self.image.seek(next_block_addr)
            next_block_addr = int.from_bytes(self.image.read(4), "big") * self.block_size
            if offset < self.block_size - 4:
                buf = self.image.read(min(self.block_size - 4, size))
                res = b"".join([res, buf[offset:]])
                offset = 0
            else:
                offset -= self.block_size - 4
            size -= self.block_size - 4
        return res

    def readdir(self, path, fh):
        files = [".", ".."]
        (_, next_block_addr, _, _, _) = self.__scan(path)
        for (kind, addr, size, time, name) in self.__read(next_block_addr):
            files.append(name)
        return files


    def __scan(self, path):
        next_block_addr = self.data_addr
        res = (0, next_block_addr, 0, 0, "") # Root dir
        for d in path[1:].split("/"):
            if d == "":
                return res
            res = (0, 0, 0, 0, "") # Not found
            for (kind, addr, size, time, name) in self.__read(next_block_addr):
                if name == d:
                    res = (kind, addr, size, time, name)
                    next_block_addr = addr
                    break
        return res

    def __read(self, next_block_addr):
        while next_block_addr != 0:
            self.image.seek(next_block_addr)
            next_block_addr = int.from_bytes(self.image.read(4), "big") * self.block_size
            offset = 4
            while offset < self.block_size:
                kind = int.from_bytes(self.image.read(1), "big")
                addr = int.from_bytes(self.image.read(4), "big") * self.block_size
                size = int.from_bytes(self.image.read(4), "big")
                time = int.from_bytes(self.image.read(8), "big")
                n = int.from_bytes(self.image.read(1), "big")
                if n == 0:
                    self.image.seek(-(1 + 4 + 4 + 8 + 1), 1) # Rewind to end of previous entry
                    break
                name = self.image.read(n).decode("utf-8")
                offset += 1 + 4 + 4 + 8 + 1 + n
                if addr > 0:
                    yield (kind, addr, size, time, name)

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('image')
    parser.add_argument('mount')
    args = parser.parse_args()
    #logging.basicConfig(level=logging.DEBUG)
    fuse = FUSE(MorosFuse(args.image), args.mount, ro=True, foreground=True, allow_other=True)
