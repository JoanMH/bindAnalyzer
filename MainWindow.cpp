//
// Created by joan on 7/03/25.
//

#include "MainWindow.h"
#include <QHeaderView>
#include <iostream>

MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent) {
    tableView = new QTableView(this);
    model = new BindModel(this);

    tableView->setModel(model);
    setCentralWidget(tableView);

    tableView->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
}

void MainWindow::loadBinds(const std::vector<Bind>& binds) {
    model->setBinds(binds);
    tableView->resizeColumnsToContents();  // 📌 Ajustar columnas al contenido
}
