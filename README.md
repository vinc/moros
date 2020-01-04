# MOROS: Omniscient Rust Operating System

```
.001  101.  .1101.  110001.  .0011.  .01011.
01'1100`11 .00  10. 01  `01 .10  10. 11'  00
10  10  11 10    11 101001' 01    01 `000.
01  00  10 00    11 00`10   00    11   `111.
10  00  10 `00  11' 00 `11. `10  10' 11   01
10  11  10  `1010'  00   01  `1100'  `11000'
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
