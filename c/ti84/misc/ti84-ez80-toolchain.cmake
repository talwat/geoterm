# WARNING: This file doesn't actually work yet.

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR ez80)

execute_process(
    COMMAND cedev-config --prefix
    OUTPUT_VARIABLE CEDEV_PREFIX
    OUTPUT_STRIP_TRAILING_WHITESPACE
)

set(META_DIR "${CEDEV_PREFIX}/meta")
set(CMAKE_C_COMPILER "${META_DIR}/ez80.alm")
set(CMAKE_CXX_COMPILER "${META_DIR}/ez80.alm")
set(CMAKE_ASM_COMPILER "${META_DIR}/commands.alm")

set(CMAKE_LINKER "${META_DIR}/ld.alm")
set(CMAKE_AR "${META_DIR}/makefile.mk")

set(CMAKE_C_FLAGS "-O2 -mmem-model=large -nostdlib")
set(CMAKE_CXX_FLAGS "-O2 -mmem-model=large -nostdlib")
set(CMAKE_EXE_LINKER_FLAGS "-T${META_DIR}/linker_script")

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unknown-warning-option")
