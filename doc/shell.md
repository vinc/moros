# MOROS Shell

## Commands

The main commands have a long name, a one-letter alias, and may have
additional common aliases.

**Alias** commands:

```
alias d delete
alias rm delete
```

**Append** to files:

```
a a.txt
append a.txt
```

**Delete** files:

```
d a.txt
del a.txt
delete a.txt
rm a.txt
```

**Copy** files:

```
c a.txt b.txt
cp a.txt b.txt
copy a.txt b.txt
```

**Move** files:

```
m a.txt b.txt
mv a.txt b.txt
move a.txt b.txt
```

**Print** strings:

```
p "Hi"
print "Hi"
echo "Hi"
```

**Read** files:

```
r a.txt
read a.txt
cat a.txt
```

**Write** files:

```
w a.txt
write a.txt
```

**Write** dirs:

```
w /usr/v/ # with a trailing slash to create a dir instead of a file
wd /usr/v # no ambiguity here so no trailing slash required
write-dir /usr/v
```

## Combiners

The `&` and `|` symbols are used only for combiners so there's no needs to
double them.

**And combiner:**

```
r a.txt & r b.txt
```

**Or combiners:**

```
r a.txt | r b.txt
```

## Pipes

The pipe symbol `|` from UNIX is replaced by `-->`, shortened to `>`, and `>`
is replaced by `--> write` or `> w` in short. An additional standard stream
stdnil(3) is added to simplify writing to `/dev/null`.

Read file A and redirect stdout(1) to stdin(0) of write file B:

```
r a.txt > w b.txt
r a.txt 1>0 w b.txt # with implicit streams
r a.txt --> w b.txt # with arrow
```

Read file A and redirect stderr(2) to stdin(0) of write file B:

```
r a.txt 2> w b.txt
r a.txt 2>0 w b.txt
```

Suppress errors by redirecting stderr(2) to stdnil(3):

```
r a.txt 2>3 w b.txt
```

Redirect stdout(1) to stdin(0) and stderr(2) to stdnil(3):

```
r a.txt > 2>3 w b.txt
r a.txt 1>0 2>3 w b.txt
```
