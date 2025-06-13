# Detect macOS and use appropriate tools
ifeq ($(shell uname),Darwin)
    CC = clang
    AR = x86_64-elf-ar
else
    CC = clang
    AR = ar
endif

CFLAGS = -target x86_64-unknown-none -ffreestanding -nostdlib -fno-stack-protector
CFLAGS += -Wall -Wextra -std=c99 -O2
CFLAGS += -Iinclude
CFLAGS += -fno-builtin -fno-omit-frame-pointer
CFLAGS += -mno-red-zone -mno-mmx -mno-sse -mno-sse2

# Source files
SRCDIR = src
SOURCES = $(wildcard $(SRCDIR)/*.c)
OBJECTS = $(SOURCES:.c=.o)

# Output
LIBDIR = lib
LIBRARY = $(LIBDIR)/libc.a

# Default target
all: $(LIBRARY)

# Create lib directory
$(LIBDIR):
	mkdir -p $(LIBDIR)

# Build library
$(LIBRARY): $(LIBDIR) $(OBJECTS)
	$(AR) rcs $@ $(OBJECTS)

# Compile source files
%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

# Clean
clean:
	rm -f $(OBJECTS)
	rm -rf $(LIBDIR)

# Install headers (optional)
install-headers:
	mkdir -p /usr/local/include/moros
	cp -r include/* /usr/local/include/moros/

# Test compilation
test: $(LIBRARY)
	$(CC) $(CFLAGS) -L$(LIBDIR) -lc -o test_program test/test.c

.PHONY: all clean install-headers test
