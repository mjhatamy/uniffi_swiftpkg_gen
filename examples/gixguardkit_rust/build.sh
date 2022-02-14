LIPO=rust-xc-universal-binary.sh
XC=xc_generator.sh

SCRIPT=$XC

./$SCRIPT libgix_guard_kit.a gix_guard_kit_rust "$(PWD)"

#export CONFIGURATION=
#export PLATFORM_NAME=iphoneos
#
#./$SCRIPT libgix_guard_kit.a gix_guard_kit_rust "$(PWD)"
#
#export CONFIGURATION=DEBUG
#export PLATFORM_NAME=macosx
#
#./$SCRIPT libgix_guard_kit.a gix_guard_kit_rust "$(PWD)"
#
#export CONFIGURATION=
#export PLATFORM_NAME=macosx
#
#./$SCRIPT libgix_guard_kit.a gix_guard_kit_rust "$(PWD)"
#
#
#CDIR=$(PWD)
#export CONFIGURATION=debug
#xcodebuild -create-xcframework -library $CDIR/target/universal/${CONFIGURATION}/osx/libgix_guard_kit.a \
#  -headers $CDIR/target/GixGuardKit   \
#  -library $CDIR/target/aarch64-apple-ios/${CONFIGURATION}/libgix_guard_kit.a \
#    -headers $CDIR/target/GixGuardKit   \
#  -library $CDIR/target/x86_64-apple-ios/${CONFIGURATION}/libgix_guard_kit.a \
#    -headers $CDIR/target/GixGuardKit   \
#  -output "gix_guard_kit.$CONFIGURATION.xcframework"
#
#export CONFIGURATION=release
#xcodebuild -create-xcframework -library $CDIR/target/universal/${CONFIGURATION}/osx/libgix_guard_kit.a \
#  -headers $CDIR/target/GixGuardKit/   \
#  -library $CDIR/target/aarch64-apple-ios/${CONFIGURATION}/libgix_guard_kit.a \
#    -headers $CDIR/target/GixGuardKit/   \
#  -library $CDIR/target/x86_64-apple-ios/${CONFIGURATION}/libgix_guard_kit.a \
#  -headers $CDIR/target/GixGuardKit/   \
#  -output "gix_guard_kit.$CONFIGURATION.xcframework"
