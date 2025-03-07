//
// Created by joan on 7/03/25.
//

#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QTableView>
#include <QApplication>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

#include "BindModel.h"


class MainWindow : public QMainWindow {
    Q_OBJECT

private:
    QTableView* tableView;
    BindModel* model;

public:
    explicit MainWindow(QWidget* parent = nullptr);
    void loadBinds(const std::vector<Bind>& binds);
};

#endif //MAINWINDOW_H
