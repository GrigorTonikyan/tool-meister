#!/bin/sh
xhost +SI:localuser:root
sudo -E /usr/bin/gufw-pkexec "$@"
xhost -SI:localuser:root
