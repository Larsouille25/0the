cmake_minimum_required(VERSION 3.1.0)

# Config of the project
project(Othebot VERSION 0.1.0)
set(EXECUTABLE_OUTPUT_PATH bin/${CMAKE_BUILD_TYPE})

# Include libs
include_directories(include)
link_directories(bin)

# Executable config
file(
        GLOB_RECURSE
        source_files
        src/*
)

add_executable(
        Othebot
        ${source_files}
)

# config the link editor
target_link_libraries(
    Othebot
    lwlogger
    
)