#include <iostream>
#include <string>
#include <fstream>
#include <filesystem>
#include <cstdlib>
#include "BindController.h"
#include <QApplication>
#include "MainWindow.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);

    MainWindow mainWindow;
    BindController controller(&mainWindow);  // 📌 Pasar la ventana al controlador

    const char* homeDir = std::getenv("HOME");
    std::string configFileName = std::string(homeDir) + "/.config/hypr/hyprland.conf";

    controller.loadBinds(configFileName);  // 📌 Llamada correcta

    mainWindow.show();

    return app.exec();
}










