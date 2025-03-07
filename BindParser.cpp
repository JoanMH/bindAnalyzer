//
// Created by joan on 7/03/25.
//

#include "BindParser.h"
#include <sstream>
#include <filesystem>
#include "utils.h"
// Procesa las líneas del archivo
std::vector<Bind> BindParser::parseFile(const std::string& filename) {
    std::vector<Bind> binds;
    std::ifstream configFile(filename);
    if (!configFile) return binds;

    std::string line;
    std::vector<std::string> tagsActivos;
    int lineNumber = 0;

    while (std::getline(configFile, line)) {
        lineNumber++;
        // Detectar líneas con TAGS
        if (line.find("#TAGS:") == 0) {
            tagsActivos.clear();
            size_t colonPos = line.find(":");

            if (colonPos != std::string::npos) {
                std::string trimmedLine = trimLeft(line.substr(6)); // Remueve "#TAGS:"
                tagsActivos = splitTags(trimmedLine);
            }
        }
        if (isBindLine(line)) {
            std::string trimmedLine = trimLeft(line.substr(5)); // Remueve "bind ="
            std::vector<std::string> modifiers = splitModifiers(trimmedLine);
            std::string key = splitKey(trimmedLine);
            std::string action = splitAction(trimmedLine);

            binds.emplace_back(modifiers, key, action, line, lineNumber, tagsActivos);
        }
    }
    return binds;
}