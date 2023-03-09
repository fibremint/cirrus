$projectRoot = Split-Path -Path $PSScriptRoot

$Env:CMAKE_ANDROID_TARGET = "aarch64"
$Env:ANDROID_API_VERSION = "33"
$Env:HOST_OS = "windows"

$ndkPrebuiltPath = "$Env:NDK_HOME\toolchains\llvm\prebuilt\$Env:HOST_OS-x86_64"
$Env:CC = "$ndkPrebuiltPath\bin\$Env:CMAKE_ANDROID_TARGET-linux-android$Env:ANDROID_API_VERSION-clang"
$Env:CXX = "$ndkPrebuiltPath\bin\$Env:CMAKE_ANDROID_TARGET-linux-android$Env:ANDROID_API_VERSION-clang++"
$Env:AR = "$ndkPrebuiltPath\bin\llvm-ar"

$Env:NDK_SYSROOT = "$ndkPrebuiltPath\sysroot"
$Env:CMAKE_GENERATOR = "Unix Makefiles"

$Env:CMAKE_TOOLCHAIN_FILE = "$projectRoot\assets\cmake\android-toolchain.cmake"