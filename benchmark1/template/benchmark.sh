#!/bin/bash
n=$1
for ((i=0; i<n; i++)); do
	make run
done
echo All done

