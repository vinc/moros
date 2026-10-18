# MOROS Shell

## Configuration

The shell will read `/ini/shell.sh` during initialization to setup its
configuration.

## Commands

The main commands have a long name, a one-letter alias, and may have
additional common aliases.

**Alias** command:

    > alias p print

<!--
**Append** to file:

    > append a.txt
    > a a.txt
-->

**Drop** file:

    > drop a.txt
    > d a.txt

**Copy** file:

    > copy a.txt b.txt
    > c a.txt b.txt

**Move** file:

    > move a.txt b.txt
    > m a.txt b.txt

**Print** string:

    > print "Hi"
    > p "Hi"

**Read** file:

    > read a.txt
    > r a.txt

**Write** file:

    > write a.txt
    > w a.txt

**Write** dir:

    > write /usr/alice/ # with a trailing slash to create a dir instead of a file
    > w /usr/alice/

**List** files in dir:

    > list /usr/alice
    > l /usr/alice

When executed without arguments, this command will list the files of the
current directory.

**Goto** dir:

    > goto /usr/alice

When executed without arguments, this command will print the current directory.

It is also possible to change the current directory by directly typing its path:

    > /usr/alice

## Combiners

The command after `and` is run only if the previous one succeeded, and the
command after `or` only if it failed. The exit code of the last command run is
available in `$?`.

**And combiner:**

    > read foo.txt and read bar.txt

**Or combiners:**

    > read foo.txt or read bar.txt

The shell also accepts commands separated by a semicolon:

    > print hello; print world
    hello
    world

## Pipes and Redirections (WIP)

A thin arrow `->` can be used for piping the output from one command to the
input of another command (TODO):

    > read foo.txt -> write bar.txt

A fat arrow `=>` can be used for redirecting directly to a file:

    > read foo.txt => bar.txt

In the following example the standard output is redirected to the null device
file while the standard error is kept:

    > time read foo.txt => /dev/null

The standard output is implied as the source of a redirection, but it is
possible to explicitly redirect a handle to another (TODO):

    > time read foo.txt [1]=>[3]

Or to redirect a handle to a file:

    > time read foo.txt [1]=> bar.txt

Or to pipe a handle to another command:

    > time read foo.txt [1]-> write bar.txt

It is possible to chain multiple redirections:

    > time read foo.txt [1]=> bar.txt [2]=> time.txt

When the arrow point to the other direction the source and destination are
swapped and the standard input is implied:

    > http <= req.txt => res.txt

Here we redirect `req.txt` to `stdin` and `stdout` to `res.txt`. If both files
are the same we can use this shortcut:

    > http <=> tmp.txt

Redirections should be declared before piping (TODO):

    > write <= req.txt => /net/http/moros.cc -> find --line href -> sort

NOTE: The following handles are available when a process is created:

- `stdin(0)`
- `stdout(1)`
- `stderr(2)`
- `stdnull(3)`

A redirection with a single arrow head will truncate its destination while
multiple heads like `=>>` will append to it.

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

    > read foo.txt ==>> bar.txt
    > read foo.txt =>> bar.txt
    > read foo.txt >> bar.txt

## Variables

- Name of the shell or the script: `$0`
- Script arguments: `$1`, `$2`, `$3`, `$4`, ...
- Exit code: `$?`
- Environment variable: `$HOME`, ...
- Shell variable: `$foo`, ...

Setting a variable in the shell is done with the following command:

    > set foo 42

    > set bar "Alice and Bob"

And accessing a variable is done with the `$` operator:

    > print $foo
    42

    > print "Hello $bar"
    Hello Alice and Bob

Unsetting a variable is done like this:

    > unset foo

Environment variables are copied to the shell when a session is started.
By convention an env var should be in uppercase and a shell var should be
lowercase. They can be changed by using `--env` or `-e`.

    > date
    2026-04-21 17:24:06 +0000

    > set --env TZ 7200

    > date
    2026-04-21 19:24:14 +0200

## Substitutions

The output of a command can be used as argument of another command:

    > print "Today is $(date)"
    Today is 2026-10-18 12:18:20 +0200

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

## Conditionals

The condition is a command, or a chain of commands with `and` and `or`, and the
first branch is taken if it succeeds. The `true` and `false` commands always
respectively succeed and fail.

    > if true do print yeah else print nope end
    yeah

    > if true do print yeah end
    yeah

The `do` keyword can be replaced by a semicolon or a newline in a script:

    #! shell

    if true
      print yeah
    else
      print nope
    end

## Loops

    > for i in 1 2 3 do print $i end
    1
    2
    3

The `do` keyword can be replaced by a semicolon or a newline in a script:

    #! shell

    for file in $(find -f "*.sh")
      print $file
      copy $file $file.old
    end

## Functions

    > def hello name do print "Hello $name" end

    > hello Alice
    Hello Alice

The `do` keyword can be replaced by a semicolon or a newline in a script:

    #! shell

    def backup file
      copy $file $file.old and print "Saved $file.old"
    end

## Reserved words

The words `and`, `def`, `do`, `else`, `end`, `for`, `if`, `in` and `or` are
reserved and cannot be used as arguments unless quoted:

    > print "end"
    end
