//
// Created by joan on 7/03/25.
//

#include "MainWindow.h"
#include <QHeaderView>
#include <iostream>
#include <QKeyEvent>
#include <set>
#include <QInputDialog>
#include <QSplitter>

#include "BindController.h"


MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent) {
    QWidget* centralWidget = new QWidget(this);
    QSplitter* splitter = new QSplitter(Qt::Horizontal, centralWidget);  // Divide horizontalmente

    tableView = new QTableView(this);
    model = new BindModel(this);
    tableView->setModel(model);
    tableView->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);

    // Crear tabla de tags
    tagTable = new QTableWidget(this);
    tagTable->setColumnCount(2);
    tagTable->setHorizontalHeaderLabels({"#", "Tag"});
    tagTable->horizontalHeader()->setSectionResizeMode(0, QHeaderView::ResizeToContents);
    tagTable->horizontalHeader()->setSectionResizeMode(1, QHeaderView::ResizeToContents);
    tagTable->setMaximumWidth(200);  // Limitar el ancho de la tabla de tags

    // Agregar ambas tablas al `QSplitter`
    splitter->addWidget(tableView);
    splitter->addWidget(tagTable);

    // Ajustar tamaños iniciales
    splitter->setStretchFactor(0, 1);  // Permitir que la tabla de binds ocupe más espacio
    splitter->setStretchFactor(1, 0);  // La tabla de tags ocupará solo lo necesario

    setCentralWidget(splitter);
    this->activateWindow();
    this->setFocus();
}

void MainWindow::loadBinds(const std::vector<Bind>& binds) {
    model->setBinds(binds);
    tableView->resizeColumnsToContents();  // Ajustar columnas al contenido


    tags = BindController::extractTags(binds);

    updateTagTable();
    filterBinds();
}

void MainWindow::filterBinds() {
    for (int row = 0; row < model->rowCount(); ++row) {
        QModelIndex index = model->index(row, 0);  // Primera columna de cada fila (puede ser otra)
        Bind bind = model->getBindAt(index.row());  // Necesitamos acceder a cada bind

        bool hasActiveTag = false;
        for (const auto& tag : bind.tags) {
            for (const auto& activeTag : tags) {
                if (QString::fromStdString(tag) == activeTag.first && activeTag.second) {
                    hasActiveTag = true;
                    break;
                }
            }
            if (hasActiveTag) break;
        }

        // Ocultar o mostrar la fila sin modificar `model`
        tableView->setRowHidden(row, !hasActiveTag);
    }
}

void MainWindow::focusInEvent(QFocusEvent *event) {
    QMainWindow::focusInEvent(event);
    setFocus();
}

void MainWindow::updateTagTable() {
    tagTable->setRowCount(tags.size());

    for (int i = 0; i < tags.size(); ++i) {
        // Celda del número
        auto* numItem = new QTableWidgetItem(QString::number(i + 1));
        numItem->setFlags(Qt::ItemIsEnabled);
        tagTable->setItem(i, 0, numItem);

        // Celda del tag con color según estado
        auto* tagItem = new QTableWidgetItem(tags[i].first);
        tagItem->setFlags(Qt::ItemIsEnabled);
        tagItem->setForeground(tags[i].second ? Qt::green : Qt::red);
        tagTable->setItem(i, 1, tagItem);
    }

    filterBinds();
}

void MainWindow::keyPressEvent(QKeyEvent *event) {
    if (event->key() == Qt::Key_D) {
        // Deseleccionar todos los tags
        for (auto &tag : tags) tag.second = false;
        updateTagTable();
    }
    else if (event->key() == Qt::Key_E) {
        // Seleccionar todos los tags
        for (auto &tag : tags) tag.second = true;
        updateTagTable();
    }
    else if (event->key() == Qt::Key_T) {
        // Esperar un número y Enter
        bool ok;
        int numTag = QInputDialog::getInt(this, "Seleccionar tag", "Introduce el número del tag:", 1, 1, tags.size(), 1, &ok);
        if (ok) {
            int index = numTag - 1;
            if (index >= 0 && index < tags.size()) {
                tags[index].second = !tags[index].second;  // Alternar estado
                updateTagTable();
            }
        }
    }
}
