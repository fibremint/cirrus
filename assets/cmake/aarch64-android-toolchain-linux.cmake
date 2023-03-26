set(CMAKE_SYSTEM_NAME Android)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

set(CMAKE_C_COMPILER $ENV{NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang)
set(CMAKE_CXX_COMPILER $ENV{NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang++)
set(CMAKE_ASM_COMPILER $ENV{NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang)

set(CMAKE_ANDROID_NDK $ENV{NDK_HOME})

set(CMAKE_ANDROID_API 33)
