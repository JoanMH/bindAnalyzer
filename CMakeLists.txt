cmake_minimum_required(VERSION 3.30)
project(bindAnalyzer)

set(CMAKE_CXX_STANDARD 17)

#Encontrar e incluir Qt6
find_package(Qt6 COMPONENTS Core Gui Widgets REQUIRED)

# 📌 Activar MOC automático para Q_OBJECT
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

#Incluir las cabeceras de Qt
include_directories(${Qt6_INCLUDE_DIRS})
add_definitions(${Qt6_DEFINITIONS})

add_executable(bindAnalyzer main.cpp
        Bind.cpp
        Bind.h
        BindParser.cpp
        BindParser.h
        utils.cpp
        utils.h
        BindController.cpp
        BindController.h
        MainWindow.cpp
        MainWindow.h
        BindModel.cpp
        BindModel.h)
# Vincular con las librerías de Qt
target_link_libraries(bindAnalyzer Qt6::Core Qt6::Gui Qt6::Widgets)
