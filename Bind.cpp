//
// Created by joan on 7/03/25.
//

#include "Bind.h"

Bind::Bind(std::vector<std::string> mods, std::string k, std::string act, std::string line,
       int num, std::vector<std::string> tgs)
   : modifiers(std::move(mods)), key(std::move(k)), action(std::move(act)), configLine(std::move(line)),
   lineNumber(num), tags(std::move(tgs)) {}

[[nodiscard]] bool Bind::matches(const std::vector<std::string>& pressedKeys) const{
    std::unordered_set<std::string> modSet(modifiers.begin(), modifiers.end());
    std::unordered_set<std::string> pressedSet(pressedKeys.begin(), pressedKeys.end());

    return modSet == pressedSet;
}