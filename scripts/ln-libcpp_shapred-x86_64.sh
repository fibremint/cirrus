ARCH=x86_64

[ ! -d "$NDK_HOME/sources/cxx-stl/llvm-libc++/libs/$ARCH" ] && mkdir -p "$NDK_HOME/sources/cxx-stl/llvm-libc++/libs/$ARCH"

ln -s $NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so \
    $NDK_HOME/sources/cxx-stl/llvm-libc++/libs/$ARCH
