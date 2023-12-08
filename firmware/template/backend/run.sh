#!/usr/bin/env bash


INPUT=$1
VECTORS=$2
DUMP_DIR=$3


# Ensure py65ucsbieee is installed
py65mon -h > /dev/null 2>&1 || {
    echo "[ERROR]: Could not run py65mon.
Please install it from ${ASSEMBLY_SRC_DIR}/tools/py65/py65ucsbieee.";
    exit 1
}

# Get Addreses for Interrupt/Reset Vectors
nmi=`   xxd -p -l 1 -seek 1 ${VECTORS}``xxd -p -l 1 -seek 0 ${VECTORS}`
reset=` xxd -p -l 1 -seek 3 ${VECTORS}``xxd -p -l 1 -seek 2 ${VECTORS}`
irq=`   xxd -p -l 1 -seek 5 ${VECTORS}``xxd -p -l 1 -seek 4 ${VECTORS}`
echo "nmi   : ${nmi}"
echo "reset : ${reset}"
echo "irq   : ${irq}"
echo "reached"
# Run py65mon
echo ${INPUT} ${reset}
py65mon --mpu 65C02 --load ${INPUT} --goto ${reset}
