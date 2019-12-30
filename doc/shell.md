# MOROS Shell

## Commands

The main commands have a long name, a one-letter alias, and may have
additional common aliases.

### Alias

Alias commands:

```
alias a alias
a d delete
alias rm delete
```

### Delete

Delete file:

```
d /usr/v/a.txt
del /usr/v/a.txt
delete /usr/v/a.txt
rm /usr/v/a.txt
```

### Copy

Copy file:

```
c /usr/v/a.txt /usr/v/b.txt
cp /usr/v/a.txt /usr/v/b.txt
copy /usr/v/a.txt /usr/v/b.txt
```

### Move

Move file:

```
m /usr/v/a.txt /usr/v/b.txt
mv /usr/v/a.txt /usr/v/b.txt
move /usr/v/a.txt /usr/v/b.txt
```

### Print

Print string:

```
p "Hi"
print "Hi"
```

### Read

Read file:

```
r /usr/v/a.txt
read /usr/v/a.txt
```

### Write

Write file:

```
w /usr/v/a.txt
write /usr/v/a.txt
```

Write dir:

```
w /usr/v/ # note the trailing slash
wd /usr/v
write-dir /usr/v
```

## Combiners

The `&` and `|` symbols are used only for combiners so there's no needs to
double them.

### And

```
r /user/v/a.txt & r /user/v/b.txt
```

### Or

```
r /user/v/a.txt | r /user/v/b.txt
```

## Pipes

The `|` symbol from UNIX is replaced by `>`, and `>` is replaced by `> w`.
An additional standard stream stdnil(3) is added to simplify writing
to `/dev/null`.

Read file A and redirect stdout(1) to stdin(0) of write file B:

```
r /user/v/a.txt > w /user/v/b.txt
r /user/v/a.txt 1>0 w /user/v/b.txt
```

Read file A and redirect stderr(2) to stdin(0) of write file B:

```
r /user/v/a.txt 2> w /user/v/b.txt
r /user/v/a.txt 2>0 w /user/v/b.txt
```

Suppress errors by redirecting stderr(2) to stdnil(3):

```
r /user/v/a.txt 2>3 w /user/v/b.txt
```

Redirect stdout(1) to stdin(0) and stderr(2) to stdnil(3):

```
r /user/v/a.txt > 2>3 w /user/v/b.txt
r /user/v/a.txt 1>0 2>3 w /user/v/b.txt
```
