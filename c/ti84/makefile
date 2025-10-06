# ----------------------------
# Makefile Options
# ----------------------------

NAME = SRLDEMO
DESCRIPTION = "CE C Toolchain Serial Demo"
COMPRESSED = NO

CFLAGS = -Wall -Wextra -Oz
CXXFLAGS = -Wall -Wextra -Oz

CEmu = /Applications/CEmu.app/Contents/MacOS/CEmu

# ----------------------------

include $(shell cedev-config --makefile)

run:
	$(CEmu) -s $(PWD)/bin/SRLDEMO.8xp --launch prgmSRLDEMO

full:
	$(MAKE)
	$(MAKE) run