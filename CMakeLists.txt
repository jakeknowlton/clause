cmake_minimum_required(VERSION 3.20)
project(clause VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler flags
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Include directories
include_directories(include)

# Common library
add_library(clause_common
    src/common/error.cpp
    src/common/source_location.cpp
)
target_include_directories(clause_common PUBLIC include)

# Lexer library
add_library(clause_lexer
    src/lexer/lexer.cpp
    src/lexer/token.cpp
)
target_link_libraries(clause_lexer PUBLIC clause_common)

# Parser library
add_library(clause_parser
    src/parser/parser.cpp
    src/parser/ast.cpp
)
target_link_libraries(clause_parser PUBLIC clause_lexer clause_common)

# Interpreter
add_library(clause_interpreter
    src/interpreter/interpreter.cpp
    src/interpreter/runtime.cpp
)
target_link_libraries(clause_interpreter PUBLIC clause_parser clause_common)

# Compiler
add_library(clause_compiler
    src/compiler/compiler.cpp
    src/compiler/codegen.cpp
)
target_link_libraries(clause_compiler PUBLIC clause_parser clause_common)

# Main executable (interpreter REPL)
add_executable(clause
    src/main.cpp
)
target_link_libraries(clause PRIVATE clause_interpreter)

# Compiler executable
add_executable(clausec
    src/compiler_main.cpp
)
target_link_libraries(clausec PRIVATE clause_compiler)

# Testing
enable_testing()

# Help CMake find GTest on macOS with Homebrew
if(APPLE)
    set(CMAKE_PREFIX_PATH "/opt/homebrew;${CMAKE_PREFIX_PATH}")
endif()

find_package(GTest REQUIRED)
if(GTest_FOUND)
    add_executable(clause_tests
        tests/lexer/lexer_test.cpp
#        tests/parser/parser_test.cpp
#        tests/interpreter/interpreter_test.cpp
#        tests/compiler/compiler_test.cpp
    )
    target_link_libraries(clause_tests
        GTest::GTest
        GTest::Main
        clause_lexer
        clause_parser
        clause_interpreter
        clause_compiler
    )
    add_test(NAME clause_tests COMMAND clause_tests)
endif()
