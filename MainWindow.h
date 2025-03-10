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
    QTableWidget* tagTable;
    std::vector<std::pair<QString, bool>> tags;  // Nombre del tag y su estado (true=activo, false=inactivo)
    void filterBinds();

public:
    explicit MainWindow(QWidget* parent = nullptr);
    void loadBinds(const std::vector<Bind>& binds);

protected:
    void keyPressEvent(QKeyEvent* event) override;  // Capturar teclas
    void focusInEvent(QFocusEvent *event) override;

private:
    void updateTagTable();
};

#endif //MAINWINDOW_H
