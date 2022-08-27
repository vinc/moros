# MOROS Shell


## Configuration

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

**And combiner:**

    > read foo.txt and read bar.txt

**Or combiners:**

    > read foo.txt or read bar.txt


## Pipes and redirections (WIP)

A thin arrow `->` can be used for piping the output from one command to the
input of another command (TODO):

    > read foo.txt -> write bar.txt

A fat arrow `=>` can be used for redirecting directly to a file:

    > read foo.txt => bar.txt

In the following example the standard output is redirected to the null device
file while the standard error is kept:

    > time read foo.txt => /dev/null

The standard output is implied as the source of a redirection, but it is
possible to explicitly redirect a file handle to another (TODO):

    > time read foo.txt [1]=>[3]

Or to redirect a file handle to a file:

    > time read foo.txt [1]=> bar.txt

Or to pipe a file handle to another command:

    > time read foo.txt [1]-> write bar.txt

It is possible to chain multiple redirections:

    > time read foo.txt [1]=> bar.txt [2]=> time.txt

When the arrow point to the other direction the source and destination are
swapped and the standard input is implied (TODO):

    > write <= req.txt => /net/http/moros.cc

Redirections should be declared before piping (TODO):

    > write <= req.txt => /net/http/moros.cc -> find --line href -> sort

NOTE: The following file handles are available when a process is created:

- `stdin(0)`
- `stdout(1)`
- `stderr(2)`
- `stdnull(3)`

<!--
NOTE: A redirection with a fat arrow will append to a file without truncating
it first. This could change in the future with with the addition of `=>>`.
-->

NOTE: Arrows can be longer, and also shorter in the case of fat arrows:

    > read foo.txt --> write bar.txt
    > read foo.txt -> write bar.txt

<!--
    > read foo.txt | write bar.txt
-->

    > read foo.txt ==> bar.txt
    > read foo.txt => bar.txt
    > read foo.txt > bar.txt

    > write bar.txt <== foo.txt
    > write bar.txt <= foo.txt
    > write bar.txt < foo.txt


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


## Tilde Expansion

The tilde character `~` is a shortcut to `$HOME` so `~/test` will be expanded
to `$HOME/test` by the shell.
