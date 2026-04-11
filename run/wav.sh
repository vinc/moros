sox "$1" -G -r 44100 -c 1 -b 8 -e unsigned-integer "$2" dither -s
