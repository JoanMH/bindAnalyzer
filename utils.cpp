//
// Created by joan on 7/03/25.
//
#include "utils.h"
#include <algorithm>
#include <sstream>

// Función para eliminar espacios al inicio de una cadena
std::string trimLeft(const std::string& str) {
    auto it = std::find_if(str.begin(), str.end(),
        [](unsigned char ch) { return !std::isspace(ch); });
    return std::string(it, str.end());
}

// Función que verifica si la línea empieza con "bind" seguido de "=" (ignorando espacios)
bool isBindLine(const std::string& line) {
    std::string trimmed = trimLeft(line);  // Elimina espacios al inicio
    if (trimmed.rfind("bind", 0) == 0) {   // ¿Empieza con "bind"?
        size_t pos = 4; // "bind" tiene 4 letras
        while (pos < trimmed.size() && std::isspace(trimmed[pos])) ++pos; // Saltar espacios
        return pos < trimmed.size() && trimmed[pos] == '='; // ¿Hay un "=" después de los espacios?
    }
    return false;
}

std::vector<std::string> splitTags(const std::string& line) {
    std::vector<std::string> modifiers;
    std::istringstream iss(line);
    std::string mod;

    while (iss >> mod) { // Extrae palabras separadas por espacios
        modifiers.push_back(mod);
    }

    return modifiers;
}

std::vector<std::string> splitModifiers(const std::string& line) {
    size_t firstComma = line.find(',');
    if (firstComma == std::string::npos) return {}; // No hay coma, retorno vacío

    std::string modifiersStr = line.substr(0, firstComma);
    std::vector<std::string> modifiers;
    std::istringstream iss(modifiersStr);
    std::string mod;

    while (std::getline(iss, mod, ' ')) { // Divide por espacios
        modifiers.push_back(mod);
    }
    return modifiers;
}

// Obtiene la tecla (entre la primera y segunda coma)
std::string splitKey(const std::string& line) {
    size_t firstComma = line.find(',');
    size_t secondComma = line.find(',', firstComma + 1);
    if (firstComma == std::string::npos || secondComma == std::string::npos) return "";

    return line.substr(firstComma + 1, secondComma - firstComma - 1);
}

// Obtiene la acción (todo después de la segunda coma)
std::string splitAction(const std::string& line) {
    size_t secondComma = line.find(',', line.find(',') + 1);
    if (secondComma == std::string::npos) return "";

    return line.substr(secondComma + 1);
}

std::string join(const std::vector<std::string>& elements, const std::string& delimiter) {
    if (elements.empty()) return "";

    std::ostringstream result;
    for (size_t i = 0; i < elements.size(); ++i) {
        if (i > 0) result << delimiter;
        result << elements[i];
    }
    return result.str();
}