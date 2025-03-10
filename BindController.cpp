//
// Created by joan on 7/03/25.
//

#include "BindController.h"
#include "BindController.h"
#include <iostream>
#include <set>

BindController::BindController(MainWindow* window) : mainWindow(window) {}

std::vector<Bind> BindController::loadBinds(const std::string& filename) {

    std::vector<Bind> binds = BindParser::parseFile(filename);

    if (mainWindow) {
        mainWindow->loadBinds(binds);
    } else {
        std::cerr << "Error: mainWindow es nullptr en BindController" << std::endl;
    }
    return binds;
}
std::vector<std::pair<QString, bool>> BindController::extractTags(const std::vector<Bind>& binds) {
    std::set<QString> uniqueTags;
    for (const auto& bind : binds) {
        for (const auto& tag : bind.tags) {
            uniqueTags.insert(QString::fromStdString(tag));
        }
    }

    std::vector<std::pair<QString, bool>> tags;
    for (const auto& tag : uniqueTags) {
        tags.emplace_back(tag, true);  // Todos los tags activos por defecto
    }

    return tags;
}

