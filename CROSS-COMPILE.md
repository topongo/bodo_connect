# Cross compiling bodo\_connect
Follow the guide below to manually install all tools and libraries to build bodo\_connect for android
with statically linked openssl.
At the end of this guid there's a section for a more simple docker build.

## OpenSSL
OpenSSL on termux is strange, I tried using dynamic linking but without success. I personally
prefer statically linking openssl for bodo\_connect. The binary is a little bigger, but the hassle
is way smaller.  
We need to get the openssl sources, then cross-compile the libraries to be statically linked to
the executable.

### Get sources
Get a stable version of openssl
```sh
git clone https://github.com/openssl/openssl --depth 1 --branch $OPENSSL_VERSION
cd openssl
```

### Configure and compile with android ndk
1. Firstly install android-ndk, with sdkmanager (`yay -S sdkmanager`) or from aur (`yay -S android-ndk`)
2. Set environment variables for ndk to work
```sh
export ANDROID_NDK_ROOT=/opt/android-ndk
export PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
```
3. Configure android architecture and [API level](https://apilevels.com/).
*Available android architectures are: `android-arm`, `android-arm64`, `android-mips`,
 `android-mip64`, `android-x86`, `android-x86_64` and `android-riscv64`.*
```sh
./Configure $OPENSSL_ANDROID_ARCH
# example: ./Configure android-arm64 -D__ANDROID_API__=34
# use `-D__ANDROID_API__=$OPENSSL_ANDROID_API` option if you want to use a specific toolchain, omit if 
you want to use the latest.
```
4. Compile
```sh
make
```

## Prepare rust toolchain
We need to add the needed android target
```sh
# example CARGO_ANDROID_TRIPLE=aarch64-linux-android
rustup toolchain install nightly
rustup target add $CARGO_ANDROID_TRIPLE
```
Check that the linker and ar executables are set in the file `.cargo/config.toml`. Example:
```toml
[target.aarch64-linux-android]
linker = "aarch64-linux-android34-clang"
```

## Compile 
We firstly need to set a bunch of env variables to make sure that cargo statically builds openssl and 
finds openssl libraries and headers.
```sh
export OPENSSL_LIB_DIR=$PWD/openssl
export OPENSSL_INCLUDE_DIR=$PWD/openssl/include/
export OPENSSL_STATIC=1
```
The we can finally compile
```sh
cargo build --release --target $CARGO_ANDROID_TRIPLE
```

## Extra: environment variables example
```sh
export OPENSSL_VERSION=openssl-3.3.2
export OPENSSL_ANDROID_ARCH=android-arm64
export OPENSSL_ANDROID_API=34
export ANDROID_NDK_ROOT=/opt/android-ndk
export PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
export OPENSSL_INCLUDE_DIR=$PWD/openssl/include
export OPENSSL_LIB_DIR=$PWD/openssl/
export OPENSSL_STATIC=1
export CARGO_ANDROID_TRIPLE=aarch64-linux-android
export MAKEFLAGS=-j$(nproc)
```

# Docker
This method uses the above steps to create a build environment capable of building every rust
project that need a statically linked openssl and must have android support.

```Dockerfile
# ...

ARG AUR_PKGS=android-ndk
# ...
ARG OPENSSL_ARCH=android-arm64
ARG OPENSSL_ANDROID_API=
ARG OPENSSL_VERSION=openssl-3.3.2
ARG CARGO_BUILD_TARGET=aarch64-linux-android

# ...
```

Copy the `Dockerfile.empty` file into `Dockerfile`, then edit the file setting the desired
options. Here's an explanation of the options:
- `AUR_PKGS`: pkgs to get and build from AUR before compiling. Usually it's some cross-compile like 
`aarch64-none-linux-gnu-gcc-12.3-bin`
- `OPENSSL_ARCH`: openssl architecture. For generic linux usage use linux-generic32 and 
linux-generic64 If using android choose the right one from this list (more details
[here](https://github.com/openssl/openssl/blob/master/NOTES-ANDROID.md)):
    - `android-arm`
    - `android-arm64`
    - `android-mips`
    - `android-mip64`
    - `android-x86`
    - `android-x86_64` 
    - `android-riscv64`
 - `OPENSSL_ANDROID_API`: (optional) android sdk/api version, refer to [this](https://apilevels.com) site
 to find the right one.
 *Note that if you leave it blank it will be used the latest available one for openssl*
 - `OPENSSL_VERSION`: which branch of the [openssl repo](https://github.com/openssl/openssl) to pull from.
 - `OPENSSL_CONFIGURE_EXTRA`: (optional) will be append after `./Configure`
 - `CARGO_BUILD_TARGET`: default cargo target, can be overrided using `cargo --target $TARGET_TRIPLE`. Keep
 in mind that if you do, you will need to install that target using `rustup target add $TARGET_TRIPLE`.

