CART=MBC5+RAM+BATTERY
# RAMSIZE 2: 64 KBit
RAMSIZE=2
ROMVER=0
TITLE=BYOTHEMESAD
LICENSEE=QQ
MFRCODE=QNVR

# ROMSIZE 3: 2 MBit
# Not used as an actual parameter!
ROMSIZE=3

# Filenames & Paths
OUTNAME=game
OUTEXT=gb
SRCPREFIX=src/
OUTPREFIX=out/
OUTFILE="${OUTPREFIX%/}/${OUTNAME}.${OUTEXT}"

# Appended to project target file to get build dir
BUILDEXT=build
# Path prefix/directory for (partial) build artifacts -- converted assets, assembled code, etc.
BUILDPREFIX=${OUTFILE}.${BUILDEXT}/

PAD_VALUE=0xFF

ASFLAGS=(-p ${PAD_VALUE} -DBUILD_CART=${CART} -DBUILD_RAMSIZE=${RAMSIZE} -DBUILD_ROMSIZE=${ROMSIZE} -Iinc/ -Isrc/ -I${BUILDPREFIX} -Wall -Wextra)
LDFLAGS=(-S romx=127,wramx=7)
#LDFLAGS=()
FIXFLAGS=(-C -m ${CART} -r ${RAMSIZE} -n ${ROMVER} -t ${TITLE} -k ${LICENSEE} -i ${MFRCODE} -p ${PAD_VALUE} -v)

