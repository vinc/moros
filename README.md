# MOROS: Omniscient Rust Operating System

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
