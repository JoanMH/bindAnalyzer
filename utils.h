//
// Created by joan on 7/03/25.
//

#ifndef UTILS_H
#define UTILS_H

#include <string>
#include <vector>

std::string trimLeft(const std::string& str);
std::vector<std::string> splitModifiers(const std::string& line);
std::string splitKey(const std::string& line);
std::string splitAction(const std::string& line);
std::vector<std::string> splitTags(const std::string& line);
bool isBindLine(const std::string& line);

// 📌 Función para unir un vector de strings en un solo string con un separador
std::string join(const std::vector<std::string>& elements, const std::string& delimiter);

#endif //UTILS_H
