//
// Created by joan on 7/03/25.
//

#include "BindController.h"
#include "BindController.h"
#include <iostream>

BindController::BindController(MainWindow* window) : mainWindow(window) {}

std::vector<Bind> BindController::loadBinds(const std::string& filename) {

    std::vector<Bind> binds = BindParser::parseFile(filename);

    if (mainWindow) {
        mainWindow->loadBinds(binds);
    } else {
        std::cerr << "Error: mainWindow es nullptr en BindController" << std::endl;
    }
    return BindParser::parseFile(filename);
}
