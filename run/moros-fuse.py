#!/usr/bin/env python

import os
from errno import ENOENT
from fuse import FUSE, FuseOSError, Operations, LoggingMixIn
from stat import S_IFDIR, S_IFREG

class MorosFuse(Operations):
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
        self.image = open(path, "rb")
        self.image_offset = 4096
        self.block_size = 512
        self.block_count = os.path.getsize(path)
        addr = self.image_offset * self.block_size
        self.image.seek(addr)
        block = self.image.read(self.block_size)
        assert block[0:8] == b"MOROS FS" # Signature
        assert block[8] == 1 # Version

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
        while next_block_addr != 0:
            self.image.seek(next_block_addr)
            next_block_addr = int.from_bytes(self.image.read(4), "big")
            offset = 4
            while offset < self.block_size:
                kind = int.from_bytes(self.image.read(1), "big")
                addr = int.from_bytes(self.image.read(4), "big") * self.block_size
                if addr == 0:
                    break
                size = int.from_bytes(self.image.read(4), "big")
                time = int.from_bytes(self.image.read(8), "big")
                n = int.from_bytes(self.image.read(1), "big")
                name = self.image.read(n).decode("utf-8")
                offset += 1 + 4 + 4 + 8 + 1 + n
                files.append(name)
        return files

    def __scan(self, path):
        dirs = path[1:].split("/")
        d = dirs.pop(0)

        bitmap_area = self.image_offset + 2
        bs = 8 * self.block_size
        total = self.block_count // self.block_size
        rest = bs * (total - bitmap_area) // bs + 1
        data_area = bitmap_area + rest // bs

        next_block_addr = data_area * self.block_size
        if d == "":
            return (0, next_block_addr, 0, 0, d)
        while next_block_addr != 0:
            self.image.seek(next_block_addr)
            next_block_addr = int.from_bytes(self.image.read(4), "big")
            offset = 4
            while offset < self.block_size:
                kind = int.from_bytes(self.image.read(1), "big")
                addr = int.from_bytes(self.image.read(4), "big") * self.block_size
                if addr == 0:
                    break
                size = int.from_bytes(self.image.read(4), "big")
                time = int.from_bytes(self.image.read(8), "big")
                n = int.from_bytes(self.image.read(1), "big")
                name = self.image.read(n).decode("utf-8")
                offset += 1 + 4 + 4 + 1 + n
                if name == d:
                    if len(dirs) == 0:
                        return (kind, addr, size, time, name)
                    else:
                        next_block_addr = addr
                        d = dirs.pop(0)
                    break
        return (0, 0, 0, 0, "")

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('image')
    parser.add_argument('mount')
    args = parser.parse_args()
    fuse = FUSE(MorosFuse(args.image), args.mount, ro=True, foreground=True, allow_other=True)
