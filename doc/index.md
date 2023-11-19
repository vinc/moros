# MOROS: Obscure Rust Operating System

![screenshot](images/moros.png)

MOROS is a hobby operating system written in Rust by
[Vincent Ollivier](https://vinc.cc).

It targets [computers](hardware.md#computers) with a x86-64 architecture and a
BIOS, so mostly from 2005 to 2020, but it also runs well on most
[emulators](hardware.md#emulators).

## Usage

MOROS is open source, you can [build it](https://github.com/vinc/moros)
or [download an image](https://github.com/vinc/moros/releases). Consult the
[manual](manual.md) to learn how to use the system.

## Features

Everything in MOROS is done from a command line interface and most programs are
rather minimalist.

It has a [shell](shell.md):

![screenshot](images/shell.png)

With a few programs like `find` that use a [regex engine](regex.md) to find
files or lines:

![screenshot](images/find.png)

It has a [calculator](calculator.md) and also a [lisp](lisp.md) interpreter:

![screenshot](images/lisp.png)

And a [text editor](editor.md):

![screenshot](images/edit.png)

It has a [network stack](network.md) with two drivers for RTL8139 and PCNET
cards:

![screenshot](images/network.png)

It has a [chess game](games.md#chess):

![chess](images/chess.png)

And the [game of life](games.md#conways-game-of-life):

![life](images/life.png)

It even has [2048](games.md#2048):

![2048](images/2048.png)

And finally it is quite customizable:

![light](images/light.png)

## Demo

You can log in to a demo with the following command using the name of the
system as a password for the guest account:

    $ ssh guest@try.moros.cc
