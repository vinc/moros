# MOROS Shell

## Commands

The main commands have a long name, a one-letter alias, and may have
additional common aliases.

<!--
**Alias** command:

    > alias d delete

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

The pipe symbol `|` from UNIX is replaced by `-->`, shortened to `>`, and `>`
is replaced by `--> write` or `> w` in short. An additional standard stream
stdnil(3) is added to simplify writing to `/dev/null`.

Read file A and redirect stdout(1) to stdin(0) of write file B:

    > r a.txt > w b.txt
    > r a.txt 1>0 w b.txt # with explicit streams
    > r a.txt --> w b.txt # with arrow

Read file A and redirect stderr(2) to stdin(0) of write file B:

    > r a.txt 2> w b.txt
    > r a.txt 2>0 w b.txt

Suppress errors by redirecting stderr(2) to stdnil(3):

    > r a.txt 2>3 w b.txt

Redirect stdout(1) to stdin(0) and stderr(2) to stdnil(3):

    > r a.txt > 2>3 w b.txt
    > r a.txt 1>0 2>3 w b.txt
