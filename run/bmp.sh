magick $1 \
  -resize 320x200^ \
  -extent 320x200 \
  -dither FloydSteinberg \
  -colors 256 \
  -depth 8 \
  -type Palette \
  -compress None \
  BMP3:$2
