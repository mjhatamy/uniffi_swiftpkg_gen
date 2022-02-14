#!/usr/bin/env bash
#set -eEuvx

function error_help() {
  ERROR_MSG="It looks like something went wrong building the Example App Universal Binary."
  echo "error: ${ERROR_MSG}"
}
trap error_help ERR

# XCode tries to be helpful and overwrites the PATH. Reset that.
PATH="$(bash -l -c 'echo $PATH')"

# This should be invoked from inside xcode, not manually
if [[ "${#}" -ne 3 ]]; then
  echo "Usage (note: only call inside xcode!):"
  echo "path/to/build-scripts/xc-universal-binary.sh <STATIC_LIB_NAME> <FFI_TARGET> <SRC_ROOT_PATH> <buildvariant>"
  exit 1
fi
# e.g. liblogins_ffi.a
STATIC_LIB_NAME=${1}
# what to pass to cargo build -p, e.g. logins_ffi
FFI_TARGET=${2}
# path to app services root
SRC_ROOT=${3}

# buildvariant from our xcconfigs
BUILDVARIANT=

RELFLAG=
RELDIR="debug"

TARGETDIR=${SRC_ROOT}/target
UNIVERSAL_BINARY_DIR=
UNIVERSAL_BINARY_URI=

# We can't use cargo lipo because we can't link to universal libraries :(
# https://github.com/rust-lang/rust/issues/55235
declare -a LIBS_ARCHS=("x86_64" "arm64")
declare -a IOS_TRIPLES=() #("x86_64-apple-ios" "aarch64-apple-ios")
declare -a IOS_SIM_TRIPLES=()
declare -a MACOS_TRIPLES=() #("aarch64-apple-darwin" "x86_64-apple-darwin")
declare -a LIPO_ARCHS=()

if [[ ! "$SRC_ROOT" = "$(PWD)" ]]; then
  cd "$SRC_ROOT" || exit
fi

HOST_ARCH=$(uname -m)

build_config() {
  CONFIGURATION=$1
  BUILDVARIANT=$(echo "${CONFIGURATION}" | tr '[:upper:]' '[:lower:]')
  if [[ "${BUILDVARIANT}" != "debug" ]]; then
    RELFLAG=--release
    RELDIR=release
  else
    RELFLAG=""
    RELDIR="debug"
  fi

  IOS_TRIPLES[0]="aarch64-apple-ios"

  IOS_SIM_TRIPLES[0]="x86_64-apple-ios"
  IOS_SIM_TRIPLES[1]="aarch64-apple-ios-sim"

  MACOS_TRIPLES[0]="x86_64-apple-darwin"
  MACOS_TRIPLES[1]="aarch64-apple-darwin"
}

build_cargo() {
  TARGET_TRIPLES=("$@")
  LIPO_ARCH_ADD_COUNTER=0
  for i in "${!TARGET_TRIPLES[@]}"; do
    if [ "${TARGET_TRIPLES[${i}]}" = "" ]; then
      continue
    fi
    env -i PATH="${PATH}" \
      "${HOME}"/.cargo/bin/cargo build --locked -p "${FFI_TARGET}" --lib ${RELFLAG} --target "${TARGET_TRIPLES[${i}]}"

    LIPO_ARCHS[${LIPO_ARCH_ADD_COUNTER}]="${TARGET_TRIPLES[${i}]}"
    LIPO_ARCH_ADD_COUNTER=$((LIPO_ARCH_ADD_COUNTER + 1))
  done


}

eval_for_platform() {
  PLATFORM_NAME=$1
  CONFIGURATION=$2
  build_config "$CONFIGURATION"

  if [[ "$PLATFORM_NAME" =~ ^(iphoneos)$ ]]; then
    echo "iphone detected $RELFLAG $CONFIGURATION"
    UNIVERSAL_BINARY_DIR=${TARGETDIR}/universal/${RELDIR}/ios
    UNIVERSAL_BINARY_URI=$UNIVERSAL_BINARY_DIR/${STATIC_LIB_NAME}

    build_cargo "${IOS_TRIPLES[@]}"
    create_fat_file "${IOS_TRIPLES[@]}"
  elif [[ "$PLATFORM_NAME" =~ ^(iphonesimulator)$ ]]; then
      echo "iphone simulator detected"
      UNIVERSAL_BINARY_DIR=${TARGETDIR}/universal/${RELDIR}/ios_sim
      UNIVERSAL_BINARY_URI=$UNIVERSAL_BINARY_DIR/${STATIC_LIB_NAME}
      build_cargo "${IOS_SIM_TRIPLES[@]}"
      create_fat_file "${IOS_SIM_TRIPLES[@]}"
  elif [[ "$PLATFORM_NAME" =~ ^(macosx|osx)$ ]]; then
    echo "macos detected"
    UNIVERSAL_BINARY_DIR=${TARGETDIR}/universal/${RELDIR}/osx
    UNIVERSAL_BINARY_URI=$UNIVERSAL_BINARY_DIR/${STATIC_LIB_NAME}
    build_cargo "${MACOS_TRIPLES[@]}"
    create_fat_file "${MACOS_TRIPLES[@]}"
  else
    echo "Unsupported platform: $PLATFORM_NAME"
    exit 1
  fi
}

need_update() {
  declare -a _LIPO_ARCHS=("$@")
  local LIPO_LIB_PATHS=()
  local UNIVERSAL_BINARY_TIMESTAMP=0
  local NEED_LIPO=0

  if [[ -f "${UNIVERSAL_BINARY_URI}" ]]; then
      UNIVERSAL_BINARY_TIMESTAMP=$(stat -f "%m" "${UNIVERSAL_BINARY_URI}")
  fi

  for LIPO_ARCH in "${_LIPO_ARCHS[@]}"; do
      if [ "$LIPO_ARCH" = "" ]; then
        continue
      fi
      LIPO_LIB_PATHS[${COUNT}]="${TARGETDIR}/$LIPO_ARCH/$RELDIR/${STATIC_LIB_NAME}"

      LIB_TIMESTAMP=1
      if [[ -f "${LIPO_LIB_PATHS[${COUNT}]}" ]]; then
          LIB_TIMESTAMP=$(stat -f "%m" "${LIPO_LIB_PATHS[${COUNT}]}")
      fi

      if [[ "$LIB_TIMESTAMP" -gt "$UNIVERSAL_BINARY_TIMESTAMP" ]]; then
        NEED_LIPO=1
      fi
      COUNT=$((COUNT + 1))
  done
  echo "$NEED_LIPO"
}

XC_FRAMEWORK_LIBS=()
XC_FRAMEWORK_LIBS_COUNTER=0;

XC_FRAMEWORK_LIBS_debug=()
XC_FRAMEWORK_LIBS_debug_COUNTER=0;

create_fat_file() {
  declare -a _LIPO_ARCHS=("$@")
  local LIPO_LIB_PATHS=()
  local COUNT=0
  local NEED_LIPO=0
  local UNIVERSAL_BINARY_TIMESTAMP=0

  NEED_LIPO=$(need_update "${_LIPO_ARCHS[@]}")

  for LIPO_ARCH in "${_LIPO_ARCHS[@]}"; do
    if [ "$LIPO_ARCH" = "" ]; then
      continue
    fi
    LIPO_LIB_PATHS[${COUNT}]="${TARGETDIR}/$LIPO_ARCH/$RELDIR/${STATIC_LIB_NAME}"
    COUNT=$((COUNT + 1))
  done

  if [[ "$RELDIR" = "debug" ]]; then
    XC_FRAMEWORK_LIBS_debug_COUNTER=$((XC_FRAMEWORK_LIBS_debug_COUNTER + 1))
    XC_FRAMEWORK_LIBS_debug[${XC_FRAMEWORK_LIBS_debug_COUNTER}]="$UNIVERSAL_BINARY_URI"
  else
    XC_FRAMEWORK_LIBS_COUNTER=$((XC_FRAMEWORK_LIBS_COUNTER + 1))
    XC_FRAMEWORK_LIBS[${XC_FRAMEWORK_LIBS_COUNTER}]="$UNIVERSAL_BINARY_URI"
  fi



  if [[ "${NEED_LIPO}" = "1" ]]; then
    if [ "$COUNT" -gt "1" ]; then
      echo "$COUNT LIPO: " "${LIPO_LIB_PATHS[@]}"
      mkdir -p "${UNIVERSAL_BINARY_DIR}"
      lipo -create -output "${UNIVERSAL_BINARY_URI}" \
        "${LIPO_LIB_PATHS[@]}"
    else
      echo "Only copying library to target directory, because only one architecture requested." "${LIPO_LIB_PATHS[@]}"
      mkdir -p "${UNIVERSAL_BINARY_DIR}"

      cp "${LIPO_LIB_PATHS[@]}" "${UNIVERSAL_BINARY_URI}"
    fi
  else
    printf "Everything is upto date %s \n" "${_LIPO_ARCHS[@]}"
  fi

}


eval_for_platform "iphoneos" "debug"
eval_for_platform "iphoneos" "release"
#
eval_for_platform "iphonesimulator" "debug"
eval_for_platform "iphonesimulator" "release"
#
eval_for_platform "macosx" "debug"
eval_for_platform "macosx" "release"


$HOME/.cargo/bin/uniffi-bindgen generate src/gix_guard.udl --language swift --out-dir "${TARGETDIR}/universal/headers"

XC_LIB_LIB_STR=""
for XC_LIB in "${XC_FRAMEWORK_LIBS_debug[@]}"; do
    if [ "XC_LIB" = "" ]; then
      continue
    fi
    XC_LIB_LIB_STR="$XC_LIB_LIB_STR -library $XC_LIB -headers ${TARGETDIR}/universal/headers"
done

xcodebuild -create-xcframework $XC_LIB_LIB_STR \
  -output ${TARGETDIR}/universal/${FFI_TARGET}_debug.xcframework


XC_LIB_LIB_STR=""
for XC_LIB in "${XC_FRAMEWORK_LIBS[@]}"; do
    if [ "XC_LIB" = "" ]; then
      continue
    fi
    XC_LIB_LIB_STR="$XC_LIB_LIB_STR -library $XC_LIB -headers ${TARGETDIR}/universal/headers"
done

xcodebuild -create-xcframework $XC_LIB_LIB_STR \
  -output ${TARGETDIR}/universal/${FFI_TARGET}.xcframework


### Move xc frameworks to Xcode lib
