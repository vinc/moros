# MOROS Shell

## Config

The shell will read `/ini/shell.sh` during initialization to setup its
configuration.

## Commands

The main commands have a long name, a one-letter alias, and may have
additional common aliases.

**Alias** command:

    > alias d delete

<!--
**Append** to file:

    > a a.txt
    > append a.txt
-->

**Delete** file:

    > d a.txt
    > del a.txt
    > delete a.txt

**Copy** file:

    > c a.txt b.txt
    > copy a.txt b.txt

**Move** file:

    > m a.txt b.txt
    > move a.txt b.txt

**Print** string:

    > p "Hi"
    > print "Hi"

**Read** file:

    > r a.txt
    > read a.txt

**Write** file:

    > w a.txt
    > write a.txt

**Write** dir:

    > write /usr/alice/ # with a trailing slash to create a dir instead of a file

**List** files in dir:

    > list /usr/alice

When executed without arguments, this command will list the files of the
current directory.

**Go to** dir:

    > goto /usr/alice

When executed without arguments, this command will print the current directory.

## Combiners (TODO)

The `&` and `|` symbols are used only for combiners so there's no needs to
double them.

**And combiner:**

    > r a.txt & r b.txt

**Or combiners:**

    > r a.txt | r b.txt

## Pipes (TODO)

The pipe symbol `|` from UNIX is replaced by a thin arrow `->`, shortened to
`>`, and the redirection symbol `>` from UNIX is replaced by a fat arrow `=>`
(see below).

Piping the standard output of a program to the `write` command to emulate a
redirection for example would be `-> write` or `> w` in short.

An additional standard stream stdnull(3) is added to simplify writing to `/dev/null`.

Examples:

Read file A and redirect stdout(1) to stdin(0) of write file B:

    > r a.txt > w b.txt
    > r a.txt 1>0 w b.txt # with explicit streams
    > r a.txt -> w b.txt # with thin arrow

Read file A and redirect stderr(2) to stdin(0) of write file B:

    > r a.txt 2> w b.txt
    > r a.txt 2>0 w b.txt

Suppress errors by redirecting stderr(2) to stdnull(3):

    > r a.txt 2>3 w b.txt

Redirect stdout(1) to stdin(0) and stderr(2) to stdnull(3):

    > r a.txt > 2>3 w b.txt
    > r a.txt 1>0 2>3 w b.txt

## Redirections

Redirecting standard IO streams can be done with a fat arrow, for example the
output of the print command can be written to a file like so:

    > print hello => /tmp/hello

Which is more efficient than doing:

    > print hello -> write /tmp/hello

## Variables

- Name of the shell or the script: `$0`
- Script arguments: `$1`, `$2`, `$3`, `$4`, ...
- Exit code: `$?`
- Process environment variable: `$HOME`, ...
- Shell environment variable: `$foo`, ...

Setting a variable in the shell environment is done with the following command:

    > set foo 42

    > set bar "Alice and Bob"

And accessing a variable is done with the `$` operator:

    > print $foo
    42

    > print "Hello $bar"
    Hello Alice and Bob

The process environment is copied to the shell environment when a session is
started. By convention a process env var should be in uppercase and a shell
env var should be lowercase.

Unsetting a variable is done like this:

    > unset foo

## Globbing

MOROS Shell support filename expansion or globbing for `*` and `?` wildcard
characters, where a pattern given in an argument of a command will be replaced
by files matching the pattern.

- `*` means zero or more chars except `/`
- `?` means any char except `/`

For example `/tmp/*.txt` will match any files with the `txt` extension inside
`/tmp`, and `a?c.txt` will match a file named `abc.txt`.
