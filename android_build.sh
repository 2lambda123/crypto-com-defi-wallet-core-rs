#!/bin/bash

if [ ! -n "$NDK_HOME" ]; then
        echo "Env NDK_HOME is empty"
        exit 1
fi

mkdir -p NDK/libs

if [ ! -f "NDK/libs/jna.aar" ]
then
        wget https://github.com/java-native-access/jna/raw/5.10.0/dist/jna.aar -P NDK/libs/ || exit 1
fi

MAKETOOL="$NDK_HOME/build/tools/make_standalone_toolchain.py"
#echo $MAKETOOL

if [ ! -x "$MAKETOOL" ]
then
        echo "Android NDK is not installed."
        exit 1
fi

uniffi-bindgen generate common/src/common.udl --config-path common/uniffi.toml --language kotlin --out-dir bindings/android || exit 1

rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android || exit 1

if [ ! -d "NDK/arm64" ]
then
        "$MAKETOOL" --api 28 --arch arm64 --install-dir NDK/arm64 2> /dev/null || exit 1
else
        echo "arm64 ndk installed."
fi

if [ ! -d "NDK/arm" ]
then
        "$MAKETOOL" --api 28 --arch arm --install-dir NDK/arm 2> /dev/null || exit 1
else
        echo "arm ndk installed."
fi

if [ ! -d "NDK/x86" ]
then
        "$MAKETOOL" --api 28 --arch x86 --install-dir NDK/x86 2> /dev/null || exit 1
else
        echo "x86 ndk installed."
fi

PATH=$PATH:`pwd`/NDK/arm64/bin cargo build --target aarch64-linux-android --release || exit 1
PATH=$PATH:`pwd`/NDK/arm64/bin cargo build --target armv7-linux-androideabi --release || exit 1
PATH=$PATH:`pwd`/NDK/arm64/bin cargo build --target i686-linux-android --release || exit 1

mkdir -p mobile_modules/android_module/dwclib/libs
cp NDK/libs/jna.aar mobile_modules/android_module/dwclib/libs/
mkdir -p mobile_modules/android_module/dwclib/src/main/jniLibs/arm64-v8a || exit 1
cp target/aarch64-linux-android/release/libdefi_wallet_core_wasm.so mobile_modules/android_module/dwclib/src/main/jniLibs/arm64-v8a/libdwc-common.so || exit 1
mkdir -p mobile_modules/android_module/dwclib/src/main/jniLibs/armeabi-v7a || exit 1
cp target/armv7-linux-androideabi/release/libdefi_wallet_core_wasm.so mobile_modules/android_module/dwclib/src/main/jniLibs/armeabi-v7a/libdwc-common.so || exit 1
mkdir -p mobile_modules/android_module/dwclib/src/main/jniLibs/x86 || exit 1
cp target/i686-linux-android/release/libdefi_wallet_core_wasm.so mobile_modules/android_module/dwclib/src/main/jniLibs/x86/libdwc-common.so || exit 1
mkdir -p mobile_modules/android_module/dwclib/src/main/java/com/defi/wallet/core/common || exit 1
cp bindings/android/com/defi/wallet/core/common/common.kt mobile_modules/android_module/dwclib/src/main/java/com/defi/wallet/core/common/ || exit 1

cd mobile_modules/android_module || exit 1
./gradlew build || exit 1
cd -
cp mobile_modules/android_module/dwclib/build/outputs/aar/dwclib-release.aar NDK/libs/dwclib.aar || exit 1
mkdir -p example/android_example/app/libs
cp NDK/libs/dwclib.aar example/android_example/app/libs/
cp NDK/libs/jna.aar example/android_example/app/libs/

echo "finish"
