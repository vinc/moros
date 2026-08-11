# MOROS Lisp Documentation

## Types

### type

    (type 42)       # => "num"
    (type "foo")    # => "str"
    (type +)        # => "fun"
    (type '(1 2 3)) # => "list"

### atom?

    (atom? 1)        # => true
    (atom? 'a)       # => true
    (atom? +)        # => true
    (atom? "test")   # => true
    (atom? '(1 2 3)) # => false

### sym?

    (sym? 'a) # => true
    (sym? 1)  # => false
    (sym? +)  # => false

### var?

    (var foo 42) # => foo
    (var? foo)   # => true
    (var? bar)   # => false

### fun?

    (fun? rev) # => true
    (fun? and) # => false

### mac?

    (mac? rev) # => false
    (mac? and) # => true

### nil?

    (nil? nil)      # => true
    (nil? '())      # => true
    (nil? '(1 2 3)) # => false
    (nil? "")       # => false
    (nil? 0)        # => false

### bool?

    (bool? true)  # => true
    (bool? false) # => true
    (bool? '())   # => false
    (bool? "")    # => false
    (bool? 0)     # => false

## Binding

### var

    (var foo 40) # => foo
    (+ foo 2)    # => 42

### mut

    (var foo 42) # => foo
    foo          # => 42
    (mut foo 10) # => 10
    foo          # => 10

### set

    (set foo 42) # => foo
    foo          # => 42
    (set foo 10) # => 10
    foo          # => 10

### let

    (let (x y) (1 2) (+ x y)) # => 3

### fun

    ((fun (x) (* x 2)) (+ 1 3)) # => 8

### mac

    (var double (mac (x) `(* ,x 2)))
    (double (+ 1 3)) # => 8

### def-fun def

    (def (rev ls)
      "Reverses the list"
      (if (nil? ls) ls
        (concat (rev (rest ls)) (cons (first ls) '()))))

### def-mac

    (def-mac (and @xs)
      (if (nil? xs) true
        (if (nil? (rest xs)) (first xs)
          `(if ,(first xs) (and ,@(rest xs)) false))))

## Quoting

### quote '

    '(1 2 3) # => (1 2 3)

### quasiquote `
### unquote ,
### unquote-splice ,@

### splice @

    ((fun (a b @c) c) '(1 2 (3))) # => (3)
    ((fun (a b @c) c) '(1 2 ()))  # => ()
    ((fun (a b @c) c) '(1 2))     # => ()

## Meta

### apply

    (apply + '(1 2 3)) # => 6
    (apply + 1 2 '())  # => 3

### load

    (load "/lib/lisp/compat.lsp")
    (car '(1 2 3)) # => 1

### doc

    (doc rev) # => "Reverses the list"

### parse

    (parse "(1 2 3)") # => (1 2 3)

### expand

    (expand (def (double x) (* x 2)))
    # => (var double (fun (x) (* x 2)))

    (expand (set foo 42))
    # => (if (var? foo) (mut foo 42) (var foo 42))

### eval

    (eval '(+ 1 2 3)) # => 6

### env

## Control Flow

### do

    (do
      (print "Compute foo")
      (compute-foo))

### if

Boolean:

    (if true 1 2)  # => 1
    (if false 1 2) # => 2
    (if true 1)    # => 1
    (if false 1)   # => ()

Truthiness (neither `false` nor `nil`):

    (if nil 1 2)   # => 2
    (if "" 1 2)    # => 1
    (if 0 1 2)     # => 1

### cond

    (set x 42)
    (cond
      ((> x 0) "positive")
      ((< x 0) "negative")
      ((= x 0) "zero")
      (true "nan"))
    # => "positive"

### case

    (case 42 (42 "yeah") (12 "nope")) # => "yeah"
    (case 12 (42 "yeah") (12 "nope")) # => "nope"
    (case 21 (42 "yeah") (12 "nope")) # => ()

### while

    (set i 0)
    (while (< (set i (+ i 1)) 100) (print
      (if (= (% i 15) 0) "fizzbuzz"
        (if (= (% i 3) 0) "fizz"
          (if (= (% i 5) 0) "buzz" i)))))

## Comparison

### eq? =

    (= 2 1)         # => false
    (= 2 2)         # => true
    (= 2 2.0)       # => true
    (= "foo" "bar") # => false
    (= "foo" "foo") # => true

### gt? >

    (> 1 2 3) # => false
    (> 3 2 1) # => true

### gte? >=

    (>= 3 2 2 1) # => true

### lt? <

    (< 2 1)   # => false
    (< 1 2)   # => true
    (< 1 2 3) # => true

### lte? <=

    (<= 1 2 2 3) # => true
    (<= 3 2 2 1) # => false

## Enumerable

### empty?

    (empty? '(1 2 3))     # => false
    (empty? '())          # => true
    (empty? "foo")        # => false
    (empty? "")           # => true
    (empty? (dict "a" 1)) # => false
    (empty? (dict))       # => true

### get

    (get '(1 2 3) 0)             # => 1
    (get '(1 2 3) 1)             # => 2
    (get "hello" 0)              # => "h"
    (get "hello" 1)              # => "e"
    (get (dict "a" 1 "b" 2) "b") # => 2

### put

    (put "hllo" 1 "e")             # => "hello"
    (put '(1 3) 1 2)               # => (1 2 3)
    (put (dict "a" 1 "c" 3) "b" 2) # => (dict "a" 1 "b" 2 "c" 3)

### len

    (len '(1 2 3)) # => 3
    (len "foo")    # => 3

### rest tail

Returns the rest of the list.

    (rest '(1 2 3)) # => (2 3)
    (rest "hello")  # => "ello"

### first head

Returns the first element of the list.

    (first '(1 2 3)) # => 1
    (first "hello")  # => "h"

### second

Returns the second element of the list.

    (second '(1 2 3)) # => 2
    (second "hello")  # => "e"

### third

Returns the third element of the list.

    (third '(1 2 3)) # => 3
    (third "hello")  # => "l"

### last

Returns the last element of the list.

    (last '(1 2 3)) # => 3
    (last "hello")  # => "o"

### slice

    (slice '(1 2 3 4 5) 0 2) # => (1 2)
    (slice '(1 2 3 4 5) 2 2) # => (3 4)
    (slice "hello" 0 2)      # => "he"
    (slice "hello" 2 2)      # => "ll"

### push

Pushes the element to the end of the list.

    (push '(1 2) 3)   # => (1 2 3)
    (push "hell" "o") # => "hello"

### contains?

    (contains? '(1 2 3) 2)  # => true
    (contains? '(1 2 3) 4)  # => false
    (contains? "hello" "e") # => true
    (contains? "hello" "a") # => false

### intersection

Returns the elements found in both lists.

    (intersection '(1 2 3) '(2 3 4)) # => (2 3)

## List

### list

    (list 1 2 3) # => (1 2 3)

### list?

    (list? '(1 2 3)) # => true
    (list? '())      # => true

### cons

    (cons 1 '(2 3)) # => (1 2 3)

### map

Applies the function to the elements of the list.

    (map first '((1 2) (3 4) (5 6))) # => (1 3 5)
    (map (fun (x) (* x 2)) '(1 2 3)) # => (2 4 6)

### fold

    (fold + 0 '(1 2 3)) # => 6
    (fold + 0 '(1 2 3)) # => -6

### reduce

Reduces the elements of the list with the function.

    (reduce + '(1 2 3)) # => 6
    (reduce - '(1 2 3)) # => -4
    (reduce + '())      # => 0

### filter

Filters the elements of the list with the function.

    (filter (fun (x) (= (% x 2) 0)) '(1 2 3 4)) # => (2 4)

### reject

Rejects the elements of the list with the function.

    (reject (fun (x) (= (% x 2) 0)) '(1 2 3 4)) # => (1 3)

### rev

Reverses the list.

    (rev '(1 2 3)) # => (3 2 1)

### sort

    (sort '(1 3 2 4 5)) # => (1 2 3 4 5)

### uniq

    (uniq '(1 2 2 3 3 4 5)) # => (1 2 3 4 5)
    (uniq '(1 2 3 3 2 4 5)) # => (1 2 3 2 4 5)

### chunks

    (chunks '(1 2 3 4) 2) # => ((1 2) (3 4))

### concat

    (concat '(1 2) '(3 4)) # => (1 2 3 4)

### range

Returns a list of numbers from start to stop excluded.

    (range 2 5) # => (2 3 4)

### nil

    nil # => ()

## Dict

### dict

    (dict "a" 1 "b" 2) # => (dict "a" 1 "b" 2)

### dict/pairs

    (dict/pairs (dict "a" 1 "b" 2)) # => (("a" 1) ("b" 2))

### dict/keys

Returns the keys of a dict.

    (dict/keys (dict "a" 1 "b" 2)) # => ("a" "b")

### dict/values

Returns the values of a dict.

    (dict/values (dict "a" 1 "b" 2)) # => (1 2)

## String

### str

    (str "foo" "bar") # => "foobar"
    (str 2)           # => "2"

### str?

    (str? "foo") # => true
    (str? "")    # => true

### bin->str

    (bin->str '(104 101 108 108 111)) # => "hello"

### str->bin

    (str->bin "hello") # => (104 101 108 108 111)

### str->num

    (str->num "123") # => 123

### str/split

    (str/split "hello world" " ") # => ("hello" "world")

### str/trim

    (str/trim "  hello world  ") # => "hello world"

### str/join

Joins the elements of the list with the string.

    (str/join '("foo" "bar") " ") # => "foo bar"

### lines

Splits the text into a list of lines.

    (lines "Hello,\nWorld!") # => ("Hello," "World!")

### words

Splits the text into a list of words.

    (words "Hello, World!") # => ("Hello," "World!")

### chars

Splits the text into a list of chars.

    (chars "hello") # => ("h" "e" "l" "l" "o")

## Number

### num?

    (num? 2)   # => true
    (num? 2.0) # => true

### bin->num

    (bin->num '(1 2 3 4 5 6 7 8) "int")      # => 72623859790382856
    (bin->num '(0 0 0 0 0 0 0 42) "int")     # => 42
    (bin->num '(63 240 0 0 0 0 0 0) "float") # => 1.0

### num->bin

    (num->bin 42)  # => (0 0 0 0 0 0 0 42)
    (num->bin 1.0) # => (63 240 0 0 0 0 0 0)

### num->str

    (num->str 1.0)   # => "1.0"
    (num->str 42)    # => "42"
    (num->str 42 2)  # => "101010"
    (num->str 42 16) # => "2A"

### num/eq?

    (num/eq? 2 2.0) # => false

### num/int

    (num/int 2.4) # => 2
    (num/int 2.6) # => 2

### num/type

    (num/type 2)         # => "int"
    (num/type 2.0)       # => "float"
    (num/type (** 2 32)) # => "int"
    (num/type (** 2 64)) # => "bigint"

### pi

    pi # => 3.141592653589793

### abs

Returns the absolute value of the number.

    (abs 2)  # => 2
    (abs -2) # => 2

### min

Returns the minimum element of the list.

    (min '(1 2 3 4 5)) # => 1

### max

Returns the maximum element of the list.

    (max '(1 2 3 4 5)) # => 5

### ceil

Returns the smallest integer greater than or equal to the number.

    (ceil -2.8) # => -2
    (ceil -2.2) # => -2
    (ceil 2.2)  # => 3
    (ceil 2.8)  # => 3

### floor

Returns the largest integer less than or equal to the number.

    (floor -2.8) # => -3
    (floor -2.2) # => -3
    (floor 2.2)  # => 2
    (floor 2.8)  # => 2

### round

Returns the nearest integer to the number.

    (round -2.8) # => -3
    (round -2.2) # => -2
    (round 2.2)  # => 2
    (round 2.8)  # => 3

## Arithmetic

### add +

    (+ 1 2 3) # => 6

### sub -

    (- 4 2)   # => 2
    (- 2 4)   # => -2
    (- 3 1.5) # => 1.5

### mul *

    (* 2 4)   # => 8
    (* 2 1.5) # => 3.0

### div /

    (/ 4 2)   # => 2
    (/ 3 2)   # => 1
    (/ 3 2.0) # => 1.5

### exp **

    (** 1.5 2) # => 2.25
    (** 2 2)   # => 4
    (** 2 1)   # => 2
    (** 2 0)   # => 1
    (** 2 32)  # => 4294967296
    (** 2 64)  # => 18446744073709551616

### rem %

    (% 7 3)   # => 1
    (% -7 3)  # => -1
    (% 7 -3)  # => 1
    (% -7 -3) # => -1

### mod

Returns the modulo of the division.

    (mod 7 3)   # => 1
    (mod -7 3)  # => 2
    (mod 7 -3)  # => -2
    (mod -7 -3) # => -1

## Trigonometric

### acos

    (acos -1.0) # => 3.141592653589793
    (acos 0.0)  # => 1.5707963267948966
    (acos 1.0)  # => 0.0

### asin

    (asin -1.0) # => -1.5707963267948966
    (asin 0.0)  # => 0.0
    (asin 1.0)  # => 1.5707963267948966

### atan

    (atan (- pi)) # => -1.2626272556789115
    (atan 0.0)    # => 0.0
    (atan pi)     # => 1.2626272556789115

### cos

    (cos (- pi)) # => -1.0
    (cos 0.0)    # => 1.0
    (cos pi)     # => -1.0

### sin

    (sin (- pi))       # => -0.00000000000000012246467991473532
    (sin (- (/ pi 2))) # => -1.0
    (sin 0.0)          # => 0.0
    (sin (/ pi 2))     # => 1.0
    (sin pi)           # => 0.00000000000000012246467991473532

### tan

    (tan pi)  # => -0.00000000000000012246467991473532
    (tan 0.0) # => 0.0

## Bit

### bit/and &

    (& 2 3 7) # => 2
    (& 1 2 3) # => 0

### bit/or |

    (| 1 2) # => 3

### bit/shl <<

    (<< 2 1)  # => 4
    (<< 2 10) # => 2048

### bit/shr >>

    (>> 4 1) # => 2

### bit/xor ^

    (^ 2 3) # => 1

## Logic

### not

    (not true)  # => false
    (not false) # => true

### and

    (and true true true) # => true
    (and true false)     # => false
    (and false false)    # => false

### or

Boolean:

    (or true true true) # => true
    (or true false)     # => true
    (or false false)    # => false

Truthiness (neither `false` nor `nil`):

    (or nil 1)          # => 1
    (or "" 1)           # => ""
    (or 0 1)            # => 0

## Regex

### regex/find

    (regex/find "\\w+" "hello world") # => (0 5)
    (regex/find "\\s" "hello world")  # => (5 6)
    (regex/find " " "hello world")    # => (5 6)

### regex/match?

Returns true if the string match the pattern.

    (regex/match? "\\d+" "42")     # => true
    (regex/match? "\\d+" "foo")    # => false
    (regex/match? "\\w+" "foo")    # => true
    (regex/match? "\\w+" "foo!")   # => true
    (regex/match? "^\\w+$" "foo!") # => false

## File

### read

Reads text file.

    (read "/dev/clk/rtc") # => "2026-05-04 13:22:31"

### write

Writes text to file.

    (write "/tmp/hello.txt" "Hello, World!\n") # => ()

### append

Appends text to file.

    (append "/tmp/hello.txt" "Lorem ipsum\n") # => ()

### dirname

Returns the given path without the filename.

    (dirname "/dev/clk/rtc") # => "/dev/clk"

### filename

Returns the filename from the given path.

    (filename "/dev/clk/rtc") # => "rtc"

### file/exists?

    (file/exists? "/dev/clk/rtc") # => true

### file/size

    (file/size "/dev/clk/rtc") # => 19

### file/open

    (file/open "/dev/clk/rtc" "r") # => 4

### file/close

    (file/close 4) # => ()

### file/read

    (set path "/dev/clk/rtc")
    (set handle (file/open path "r"))
    (set length (file/size path))
    (set buffer (file/read handle length))
    (bin->str buffer) # => "2026-05-04 13:22:31"
    (file/close handle)

### file/write

    (set path "/tmp/hello.txt")
    (set handle (file/open path "w"))
    (set buffer (str->bin "Hello, World!\n"))
    (file/write handle buffer) # => 14
    (file/close handle)

### read-bin
### write-bin
### append-bin
### read-line
### read-char

## Network

### host

    (host "moros.cc") # => "172.67.165.252"

### socket/connect

Daytime Protocol:

    (set socket (socket/connect "tcp" (host "time.nist.gov") 13))
    (set res (str/trim (bin->str (file/read socket 64))))
    (file/close socket)
    (slice (words res) 1 2) # => ("26-05-08" "15:28:50")

Hypertext Transfer Protocol:

    # Connection
    (set socket (socket/connect "tcp" (host "moros.cc") 80))

    # Request
    (set req (str/join (list
      "GET / HTTP/1.1"
      "Host: moros.cc"
      "Connection: close"
      "" "") "\r\n"))
    (file/write socket (str->bin req))

    # Response
    (set mtu (file/size "/dev/net/tcp"))
    (set res (bin->str (file/read socket mtu)))
    (file/close socket)
    (str/trim (first (lines res))) # => "HTTP/1.1 200 OK"

### socket/accept

    (socket/accept socket) # => "10.0.2.2"

Echo Server:

    (set mtu (file/size "/dev/net/tcp"))
    (set socket (socket/listen "tcp" 80))
    (while true (do
      (if (socket/accept socket) (do
        (set buf (file/read socket mtu))
        (file/write socket buf)
        (file/close socket)
        (set socket (socket/listen "tcp" 80))))))

TODO: In the future `socket/accept` will return `(handle address port)` or
`()` without blocking when the syscall is updated to keep the listening socket
open when a connection is closed.

### socket/listen

    (socket/listen "tcp" 80) # => 4

## System

### print

Prints expression to stdout with a newline.

    (print "Hello, World!") # => ()

### date

    (date 0)          # => "1970-01-01 00:00:00"
    (date 1234567890) # => "2009-02-13 23:31:30"

### sleep

    (sleep 0.864) # => ()

### sh

    (sh "print hello") # => 0

### sh->bin

    (sh->bin "print hello") # => (104 101 108 108 111 10)

### sh->str

Returns the output of the command.

    (sh->str "print hello") # => "hello"

### clock/boot

Returns the number of seconds since boot.

    (clock/boot) # => 1152.069566

### clock/epoch

Returns the number of seconds since epoch.

    (clock/epoch) # => 1778244455.765883
