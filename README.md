# MOROS: Omniscient Rust Operating System

```
.100  110.  .1100.  111110.  .1001.  .01000.
00'1001`11 .11  01. 00  `00 .10  00. 10'  11
01  00  10 10    00 001101' 01    00 `100.
10  01  10 01    11 01`00   01    11   `100.
00  01  01 `00  11' 10 `11. `00  11' 01   00
11  10  10  `1001'  00   01  `0110'  `01101'
```

MOROS is a toy operating system written in Rust.

## Usage

Install tools:

    curl https://sh.rustup.rs -sSf | sh
    rustup install nightly
    rustup default nightly
    cargo install cargo-xbuild bootimage

Run with:

    cargo xrun


## LICENSE

This project started from the [seventh post][1] of the second edition of
[Writing an OS in Rust][2] by Philipp Oppermann.

MOROS is released under MIT.

[1]: https://github.com/phil-opp/blog_os/tree/post-07
[2]: https://os.phil-opp.com
