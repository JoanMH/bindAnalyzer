//
// Created by joan on 7/03/25.
//

#ifndef BIND_H
#define BIND_H

#include <vector>
#include <string>
#include <unordered_set>

class Bind {
public:
    std::vector<std::string> tags; // Lista de hasta 3 tags
    std::vector<std::string> modifiers; // SUPER, ALT, CTRL, etc.
    std::string key; // La tecla principal
    std::string action; // Acción ejecutada por el bind
    std::string configLine; // Línea original del archivo de configuration
    int lineNumber;

    Bind() = default;
    Bind(std::vector<std::string> mods, std::string k, std::string act, std::string line,
        int num, std::vector<std::string> tgs);

    [[nodiscard]] bool matches(const std::vector<std::string>& pressedKeys) const;
};


#endif //BIND_H
