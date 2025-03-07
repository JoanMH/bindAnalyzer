//
// Created by joan on 7/03/25.
//

#ifndef BINDPARSER_H
#define BINDPARSER_H

#include <vector>
#include <string>
#include <fstream>
#include "Bind.h"

class BindParser {
public:
    static std::vector<Bind> parseFile(const std::string& filename);
};



#endif //BINDPARSER_H
