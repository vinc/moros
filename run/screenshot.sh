if [ $# -eq 0 ]; then
    echo "Usage: screenshot.sh <name>"
    exit 1
fi
nc -N 127.0.0.1 7777 <<< "screendump $1.ppm"
convert "$1.ppm" "$1.png"
optipng "$1.png"
feh "$1.png"
