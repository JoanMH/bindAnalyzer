#include <iostream>
#include <vector>
#include <string>
#include <unordered_set>
#include <fstream>
#include <filesystem>
#include <cstdlib>
#include <algorithm>

class Bind {
public:
    std::vector<std::string> tags; // Lista de hasta 3 tags
    std::vector<std::string> modifiers; // SUPER, ALT, CTRL, etc.
    std::string key; // La tecla principal
    std::string action; // Acción ejecutada por el bind
    std::string configLine; // Línea original del archivo de configuration
    int lineNumber;

    Bind(std::vector<std::string> mods, std::string k, std::string act, std::string line,
        int num, std::vector<std::string> tgs)
    : modifiers(std::move(mods)), key(std::move(k)), action(std::move(act)), configLine(std::move(line)),
    lineNumber(num), tags(std::move(tgs)) {}


    [[nodiscard]] bool matches(const std::vector<std::string>& pressedKeys) const {
        std::unordered_set<std::string> modSet(modifiers.begin(), modifiers.end());
        std::unordered_set<std::string> pressedSet(pressedKeys.begin(), pressedKeys.end());

        return modSet == pressedSet;
    }
};

//Declaracion de funciones
std::vector<Bind> parseBindLines(std::ifstream& configFile);
bool isBindLine(const std::string& line);
std::string trimLeft(const std::string& str);
std::vector<std::string> splitModifiers(const std::string& line);
std::string splitKey(const std::string& line);
std::string splitAction(const std::string& line);
std::vector<std::string> splitTags(const std::string& line);
//Fin declaración de funciones

int main() {
    //auto lang = "C++";
    const char* homeDir = std::getenv("HOME");

    std::string configFileName = std::string(homeDir) + "/.config/hypr/hyprland.conf";
    //primero leeremos el fichero de configuracion
    if (std::filesystem::exists(configFileName)) {
        std::ifstream configFile(configFileName, std::ios::in);
        if (!configFile.is_open()) {
            std::cerr << "Error opening file " << configFileName << std::endl;
            return 1;
        }
        std::vector<Bind> binds = parseBindLines(configFile);
        configFile.close();

        std::cout << "\n===== Lista de Binds Detectados =====\n";
        for (const auto& bind : binds) {
            std::cout << "Línea " << bind.lineNumber << ": " << bind.configLine << " | Modificadores: ";
            for (const auto& mod : bind.modifiers) {
                std::cout << mod << " ";
            }
            std::cout << "|TAGS : ";
            for (const auto& tag : bind.tags) {
                std::cout << tag << " ";
            }
            std::cout << "| Tecla: " << bind.key << " | Acción: " << bind.action << std::endl;
        }
        std::cout << "=====================================" << std::endl;
    } else {
        std::cout << "Fichero no encontrado" << std::endl;
    }
    return 0;
}

// Procesa las líneas del archivo
std::vector<Bind> parseBindLines(std::ifstream& configFile) {
    std::vector<Bind> binds;
    std::string line;
    std::vector<std::string> tagsActivos;
    int lineNumber = 0;

    while (std::getline(configFile, line)) {
        lineNumber++;
        // Detectar líneas con TAGS
        if (line.find("#TAGS:") == 0) {
            std::cout << "HE ENCONTRADO INA LIMEA TAGS!!!!!!!!!!!!!!" << std::endl;
            tagsActivos.clear();
            size_t colonPos = line.find(":");

            if (colonPos != std::string::npos) {
                std::string trimmedLine = trimLeft(line.substr(6)); // Remueve "#TAGS:"
                tagsActivos = splitTags(trimmedLine);
                std::cout << "[TAGS TEST ";
                for (const auto& item : tagsActivos) {
                    std::cout << item << " ";
                }
                std::cout << "]" << std::endl;
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






