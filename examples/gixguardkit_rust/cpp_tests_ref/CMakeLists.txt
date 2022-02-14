cmake_minimum_required(VERSION 3.16)
project(cpp_tests_ref C ASM)

set(CMAKE_CXX_STANDARD 14)
#set(CMAKE_C_STANDARD 99)
#set(CMAKE_C_FLAGS "${CMAKE_ENABLE_C99}")
#set(CMAKE_C_STANDARD_REQUIRED TRUE)
#set(CMAKE_ASM_FLAGS "${CFLAGS} ${CMAKE_C_FLAGS} -x assembler-with-cpp")

#if (POLICY CMP0025)
#    cmake_policy(SET CMP0025 NEW)
#endif ()

set(WireGuardKitC_Files
        WireGuardKitC/key.c
        WireGuardKitC/key.h
        WireGuardKitC/WireGuardKitC.h
        WireGuardKitC/x25519.h
        WireGuardKitC/x25519.c
        )

add_executable(${CMAKE_PROJECT_NAME} main.c
        ${WireGuardKitC_Files}
        )
target_link_libraries(${CMAKE_PROJECT_NAME}
        ${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/libgix_guard_kit.a
        )
#set_property(TARGET ${CMAKE_PROJECT_NAME} PROPERTY C_STANDARD 99)
#target_compile_features(${CMAKE_PROJECT_NAME} PUBLIC c_std_99)
