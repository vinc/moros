set -e

file="squashware.zip"
base="https://github.com/fragglet/squashware/releases/download"
curl -sSL "$base/squashware-1.3/squashware-silent-1.3.zip" -z "$file" -o "$file"
unzip "$file"
mv newdoom1_silent.wad squashware.wad
rm "$file"
pigz --zlib --best --suffix .z squashware.wad

file="doom1.wad"
base="https://distro.ibiblio.org/slitaz/sources/packages/d"
curl -sSL "$base/$file" -z "$file" -o "$file"
pigz --zlib --best --suffix .z "$file"
