cmake_minimum_required(VERSION 3.16)
project(rust_cpp_test)

set(CMAKE_CXX_STANDARD 14)

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/../bindings/c")

add_executable(rust_cpp_test main.cpp
        )
#target_include_directories(rust_cpp_test PUBLIC
#        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../bindings/c>)
target_link_libraries(rust_cpp_test
        ${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/libgix_guard_kit.a
        -lpthread
        ${CMAKE_DL_LIBS}
        )
