################################################################################
#################################### TARGET ####################################
################################################################################

CART=MBC5+RAM+BATTERY
ROMVER=0
TITLE=BYOTHEMESAD
LICENSEE=QQ
MFRCODE=QNVR

## $0148 -- ROM size
## Note that the target ROM size is determined automatically.
## This value is used with the `rgblink -S` bank scramble option, and
## is accessible at assembly time using the `BUILD_ROMSIZE` constant.
## $03: 2 MBit ~ 256 MiB ~ 16 banks
ROMSIZE=3

## $0149 -- RAM size
## $02: 64 KBit
RAMSIZE=2

PAD_VALUE=0xFF


################################################################################
#################################### OUTPUT ####################################
################################################################################

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


################################################################################
################################# BUILD TOOLS ##################################
################################################################################

ASFLAGS=(-p ${PAD_VALUE} -DBUILD_CART=${CART} -DBUILD_RAMSIZE=${RAMSIZE} -DBUILD_ROMSIZE=${ROMSIZE} -Iinc/ -Isrc/ -I${BUILDPREFIX} -Wall -Wextra)
LDFLAGS=(-S romx=$(( (2 << ROMSIZE) - 1 )),wramx=7)
FIXFLAGS=(-C -m ${CART} -r ${RAMSIZE} -n ${ROMVER} -t ${TITLE} -k ${LICENSEE} -i ${MFRCODE} -p ${PAD_VALUE} -v)


################################################################################
################################ DEBUG/RUN/TEST ################################
################################################################################

## Mesen(2) emulator command
MESENEXE=Mesen
## Command args to use when running the Mesen emulator (excluding the rom to load)
MESENARGS=()


# vim: ft=bash

