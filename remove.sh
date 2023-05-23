#!/bin/sh
tree -I target -I "index.html" -I "remove.sh" -I ".git" -f -i -L 1 --noreport | grep -v "^.$" | xargs rm -r
