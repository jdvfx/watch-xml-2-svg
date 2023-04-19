find . -name "*.svg" | xargs -n 1 | cut -d '/' -f 2  | cut -d '.' -f 1 | xargs -I % convert %.svg %.png
