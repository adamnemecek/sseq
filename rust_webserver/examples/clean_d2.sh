#!/bin/sh

# This converts a d2 file generated by ext to one understood by our charts.rs

sed -i 's/ //g' $1
sed -i 's/d_2x_(//' $1
sed -i 's/)=\[/,/' $1
sed -i 's/]//' $1
