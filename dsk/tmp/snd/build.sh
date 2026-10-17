set -e

# Frederic Chopin - Nocturne in B-flat minor, Op. 9, No. 1
# Performed by Olga Gurevich for Musopen Chopin Project
# Public Domain (CC0 1.0)
wget -c https://archive.org/download/musopen-chopin/NocturneOp.9No.1InBFlatMinor.mp3 -O chopin.mp3
sox chopin.mp3 -G -r 44100 -c 1 -b 8 -e unsigned-integer -t raw chopin.pcm trim 0 18 dither -s
sox chopin.mp3 -G -r 8000 -c 2 -b 16 -e signed-integer -t wav chopin.wav trim 0 18 dither -s
sh ../../../run/deflate.sh chopin.pcm
sh ../../../run/deflate.sh chopin.wav
