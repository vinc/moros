(var stdin 0)
(var stdout 1)
(var stderr 2)

# Read

(def (read-binary path)
  "Reads binary file"
  (do
    (var size (file/size path))
    (var file (file/open path "r"))
    (var data (file/read file size))
    (file/close file)
    data))

(def (read path)
  "Reads text file"
  (binary->string (read-binary path)))

# Write

(def (write-binary path data)
  "Writes binary to file"
  (do
    (var file (file/open path "w"))
    (file/write file data)
    (file/close file)))

(def (write path text)
  "Writes text to file"
  (write-binary path (string->binary text)))

# Append

(def (append-binary path data)
  "Appends binary to file"
  (do
    (var file (file/open path "a"))
    (file/write file data)
    (file/close file)))

(def (append path text)
  "Appends text to file"
  (append-binary path (string->binary text)))

# Console

(def (read-line)
  "Reads line from the console"
  (string/trim (binary->string (file/read stdin 256))))

(def (read-char)
  "Reads char from the console"
  (binary->string (file/read stdin 4)))

(def (p exp)
  "Prints expression to the console"
  (do
    (file/write stdout (string->binary (string exp)))
    '()))

(def (print exp)
  "Prints expression to the console with a newline"
  (p (string exp "\n")))

# Clocks

(def (uptime)
  "Returns the current value of the uptime clock"
  (binary->number (read-binary "/dev/clk/uptime") "float"))

(def (realtime)
  "Returns the current value of the realtime clock"
  (binary->number (read-binary "/dev/clk/realtime") "float"))

# Path

(def (filename path)
  "Returns the filename from the given path"
  (last (str/split path "/")))

(def (dirname path)
  "Returns the given path without the filename"
  (str/join (rev (rest (rev (str/split path "/")))) "/"))
