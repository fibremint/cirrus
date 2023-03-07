TARGET=aarch64

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PROJECT_ROOT="$SCRIPT_DIR/.."

export NDK_HOME="$HOME/Android/Sdk/ndk/25.1.8937393"
export NDK_PREBUILT_PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

export CC="$NDK_PREBUILT_PATH/$TARGET-linux-android33-clang"
export CXX="$NDK_PREBUILT_PATH/$TARGET-linux-android33-clang++"
export AR="$NDK_PREBUILT_PATH/llvm-ar"

export CMAKE_TOOLCHAIN_FILE="$PROJECT_ROOT/assets/cmake/$TARGET-android-toolchain-linux.cmake"
