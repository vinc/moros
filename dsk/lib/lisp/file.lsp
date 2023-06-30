(var stdin 0)
(var stdout 1)
(var stderr 2)

# Read

(def (read-file-binary path)
  "Read binary file"
  (do
    (var size (file:size path))
    (var file (file:open path "r"))
    (var data (file:read file size))
    (file:close file)
    data))

(def (read-file path)
  "Read text file"
  (binary->string (read-file-binary path)))

# Write

(def (write-file-binary path data)
  "Write binary to file"
  (do
    (var file (file:open path "w"))
    (file:write file data)
    (file:close file)))

(def (write-file path text)
  "Write text to file"
  (write-file-binary path (string->binary text)))

# Append

(def (append-file-binary path data)
  "Append binary to file"
  (do
    (var file (file:open path "a"))
    (file:write file data)
    (file:close file)))

(def (append-file path text)
  "Append text to file"
  (append-file-binary path (string->binary text)))

# Console

(def (read-line)
  "Read line from the console"
  (string:trim (binary->string (file:read stdin 256))))

(def (read-char)
  "Read char from the console"
  (binary->string (file:read stdin 4)))

(def (p exp)
  "Print expression to the console"
  (do
    (file:write stdout (string->binary (string exp)))
    '()))

(def (print exp)
  "Print expression to the console with a newline"
  (p (string exp "\n")))

# Special

(def (uptime)
  (binary->number (read-file-binary "/dev/clk/uptime") "float"))

(def (realtime)
  (binary->number (read-file-binary "/dev/clk/realtime") "float"))
