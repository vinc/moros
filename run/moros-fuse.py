#!/usr/bin/env python

import logging
import os
from time import time
from errno import ENOENT
from fuse import FUSE, FuseOSError, Operations, LoggingMixIn
from stat import S_IFDIR, S_IFREG

BLOCK_SIZE = 512
SUPERBLOCK_ADDR = 4096 * BLOCK_SIZE
BITMAP_ADDR = SUPERBLOCK_ADDR + 2 * BLOCK_SIZE

class MorosFuse(LoggingMixIn, Operations):
    chmod = None
    chown = None
    readlink = None
    rename = None
    rmdir = None
    symlink = None
    truncate = None
    unlink = None
    utimens = None

    def __init__(self, path):
        self.block_size = BLOCK_SIZE
        self.image = open(path, "r+b")
        self.image.seek(SUPERBLOCK_ADDR)
        superblock = self.image.read(self.block_size)
        assert superblock[0:8] == b"MOROS FS" # Signature
        assert superblock[8] == 1 # Version
        assert self.block_size == 2 << (8 + superblock[9])
        self.block_count = int.from_bytes(superblock[10:14], "big")
        self.alloc_count = int.from_bytes(superblock[14:18], "big")

        bs = 8 * self.block_size # number of bits per bitmap block
        #print("bs = %d" % bs)
        total = self.block_count
        #print("total = %d" % total)
        rest = (total - (BITMAP_ADDR // self.block_size)) * bs // (bs + 1)
        #print("rest = %d" % rest)
        self.data_addr = BITMAP_ADDR + (rest // bs) * self.block_size
        #print("superblock addr = 0x%06x" % SUPERBLOCK_ADDR)
        #print("bitmap addr     = 0x%06x" % BITMAP_ADDR)
        #print("data addr       = 0x%06x" % self.data_addr)
        addr = self.__next_free_addr()
        #print("next free addr  = 0x%06x" % addr)
        #print("is_alloc(0x%06x) -> %s" % (self.data_addr, self.__is_alloc(self.data_addr)))
        #print("is_alloc(0x%06x) -> %s" % (addr - self.block_size, self.__is_alloc(addr - self.block_size)))
        #print("is_alloc(0x%06x) -> %s" % (addr, self.__is_alloc(addr)))
        self.__alloc(addr)
        #print("is_alloc(0x%06x) -> %s" % (addr, self.__is_alloc(addr)))
        self.__free(addr)
        #print("is_alloc(0x%06x) -> %s" % (addr, self.__is_alloc(addr)))

    def __next_free_addr(self):
        for bitmap_addr in range(BITMAP_ADDR, self.data_addr):
            self.image.seek(bitmap_addr)
            bitmap = self.image.read(self.block_size)
            for i in range(self.block_size):
                byte = bitmap[i]
                #print("byte = {:#010b}".format(byte))
                for bit in range(0, 8):
                    if (byte >> bit & 1) == 0:
                        block = (bitmap_addr - BITMAP_ADDR) * self.block_size * 8 + i * 8 + bit
                        return self.data_addr + block * self.block_size

    def __is_alloc(self, addr):
        block = (addr - self.data_addr) // self.block_size
        bitmap_addr = BITMAP_ADDR + (block // (self.block_size * 8))
        pos = bitmap_addr + (block // 8)
        self.image.seek(pos)
        byte = int.from_bytes(self.image.read(1), "big")
        bit = block % 8
        #print("bitmap_addr     = 0x%06x" % bitmap_addr)
        #print("pos             = 0x%06x" % pos)
        #print("block = %d" % block)
        #print("byte = {:#010b}".format(byte))
        #print("bit = %d" % bit)
        return (byte >> bit & 1) == 1

    def __alloc(self, addr):
        #print("\nalloc")
        block = (addr - self.data_addr) // self.block_size
        bitmap_addr = BITMAP_ADDR + (block // (self.block_size * 8))
        self.image.seek(bitmap_addr + (block // 8))
        byte = int.from_bytes(self.image.read(1), "big")
        self.image.seek(-1, 1)
        #print("byte = {:#010b}".format(byte))
        bit = block % 8
        byte |= (1 << bit)
        #print("byte = {:#010b}".format(byte))
        self.image.write(bytes([byte]))

    def __free(self, addr):
        #print("\nfree")
        block = (addr - self.data_addr) // self.block_size
        bitmap_addr = BITMAP_ADDR + (block // (self.block_size * 8))
        self.image.seek(bitmap_addr + (block // 8))
        byte = int.from_bytes(self.image.read(1), "big")
        self.image.seek(-1, 1)
        #print("byte = {:#010b}".format(byte))
        bit = block % 8
        byte &= ~(1 << bit)
        #print("byte = {:#010b}".format(byte))
        self.image.write(bytes([byte]))

    def destroy(self, path):
        self.image.close()
        return

    def getattr(self, path, fh=None):
        (kind, addr, size, time, name) = self.__scan(path)
        #print("DEBUG: getattr('%s') -> (%s, 0x%s, %d, %d, '%s')" % (path, kind, addr, size, time, name))
        if addr == 0:
            raise FuseOSError(ENOENT)
        mode = S_IFDIR | 0o755 if kind == 0 else S_IFREG | 0o644
        return { "st_atime": 0, "st_mtime": time, "st_uid": 0, "st_gid": 0, "st_mode": mode, "st_size": size }

    def read(self, path, size, offset, fh):
        (kind, next_block_addr, size, time, name) = self.__scan(path)
        res = b""
        while next_block_addr != 0 and size > 0:
            self.image.seek(next_block_addr)
            next_block_addr = int.from_bytes(self.image.read(4), "big") * self.block_size
            if offset < self.block_size - 4:
                buf = self.image.read(max(0, min(self.block_size - 4, size)))
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

    def mkdir(self, path, mode):
        self.create(path, S_IFDIR | mode)

    def create(self, path, mode):
        (path, _, name) = path.rpartition("/")
        entries = self.readdir(path + "/", 0)
        pos = self.image.tell()

        # Update parent dir size
        (_, _, parent_size, _, parent_name) = self.__scan(path)
        parent_size = (1 + 4 + 4 + 8 + 1) * len(entries) + len("".join(entries))
        self.image.seek(-(4 + 8 + 1 + len(parent_name)), 1)
        self.image.write(parent_size.to_bytes(4, "big"))

        kind = int((mode & S_IFDIR) != S_IFDIR)
        size = 0
        addr = self.__next_free_addr()
        self.__alloc(addr)

        self.image.seek(pos)
        self.image.write(kind.to_bytes(1, "big"))
        self.image.write((addr // self.block_size).to_bytes(4, "big"))
        self.image.write(size.to_bytes(4, "big"))
        self.image.write(int(time()).to_bytes(8, "big"))
        self.image.write(len(name).to_bytes(1, "big"))
        self.image.write(name.encode("utf-8"))
        return 0

    def write(self, path, data, offset, fh):
        (_, addr, size, _, name) = self.__scan(path)
        pos = self.image.tell()
        print(" --> pos=%s addr=%s size=%d name='%s'" % (pos, addr, size, name))

        n = self.block_size - 4 # Space available for data in blocks
        j = size % n # Start of space available in last block

        # Update file size
        self.image.seek(-(4 + 8 + 1 + len(name)), 1)
        size = max(size, offset + len(data))
        self.image.write(size.to_bytes(4, "big"))

        (_, addr, size, _, name) = self.__scan(path)
        pos = self.image.tell()
        print(" --> pos=%s addr=%s size=%d name='%s'" % (pos, addr, size, name))

        for i in range(0, offset, n):
            self.image.seek(addr)
            next_addr = int.from_bytes(self.image.read(4), "big") * self.block_size
            if i + n >= offset:
                self.image.seek(addr + 4 + j)
                self.image.write(data[0:(n - j)])
            if next_addr == 0:
                next_addr = self.__next_free_addr()
                self.__alloc(next_addr)
                self.image.seek(addr)
                self.image.write((next_addr // self.block_size).to_bytes(4, "big"))
            addr = next_addr

        for i in range(n - j if j > 0 else 0, len(data), n):
            next_addr = 0
            if i + n < len(data): # TODO: check for off by one error
                next_addr = self.__next_free_addr()
                self.__alloc(next_addr)
            self.image.seek(addr)
            self.image.write((next_addr // self.block_size).to_bytes(4, "big"))
            self.image.write(data[i:min(i + n, len(data))])
            addr = next_addr

        print("\x1b[0m")
        return len(data)

    def __scan(self, path):
        #print("DEBUG: __scan('%s')" % path)
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
        #print("DEBUG: __read(0x%x)" % next_block_addr)
        while next_block_addr != 0:
            self.image.seek(next_block_addr)
            #print("DEBUG: __read begin block -> pos=0x%x" % self.image.tell())
            next_block_addr = int.from_bytes(self.image.read(4), "big") * self.block_size
            offset = 4
            while offset < self.block_size:
                #print("DEBUG: __read begin entry -> pos=0x%x (%d)" % (self.image.tell(), offset))
                kind = int.from_bytes(self.image.read(1), "big")
                addr = int.from_bytes(self.image.read(4), "big") * self.block_size
                size = int.from_bytes(self.image.read(4), "big")
                time = int.from_bytes(self.image.read(8), "big")
                n = int.from_bytes(self.image.read(1), "big")
                if n == 0:
                    self.image.seek(-(1 + 4 + 4 + 8 + 1), 1) # Rewind to end of previous entry
                    break
                name = self.image.read(n).decode("utf-8")
                #print("DEBUG: __read end entry   -> pos=0x%x" % self.image.tell())
                #print("DEBUG:                    `-------------> (%s, 0x%s, %d, %d, '%s')" % (kind, addr, size, time, name))
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
    fuse = FUSE(MorosFuse(args.image), args.mount, ro=False, foreground=True, allow_other=True)
