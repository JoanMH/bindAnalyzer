//
// Created by joan on 7/03/25.
//

#ifndef BINDCONTROLLER_H
#define BINDCONTROLLER_H

#include <vector>
#include <string>
#include "Bind.h"
#include "BindParser.h"
#include "MainWindow.h"

class BindController {
public:

private:
    MainWindow* mainWindow;

public:
    explicit BindController(MainWindow* window);
    std::vector<Bind> loadBinds(const std::string& filename);
    static std::vector<std::pair<QString, bool>> extractTags(const std::vector<Bind>& binds);
};

#endif //BINDCONTROLLER_H
