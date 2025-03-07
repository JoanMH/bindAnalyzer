#include "BindModel.h"
#include "utils.h"

BindModel::BindModel(QObject* parent) : QAbstractTableModel(parent) {}

void BindModel::setBinds(const std::vector<Bind>& newBinds) {
    beginResetModel();
    binds = newBinds;
    endResetModel();
}

int BindModel::rowCount(const QModelIndex& parent) const {
    return static_cast<int>(binds.size());
}

int BindModel::columnCount(const QModelIndex& parent) const {
    return 5; // Modificadores, Tecla, Acción, Línea, Tags
}

QVariant BindModel::data(const QModelIndex& index, int role) const {
    if (role == Qt::DisplayRole) {
        const Bind& bind = binds[index.row()];
        switch (index.column()) {
            case 1: return QString::fromStdString(bind.key);
            case 2: return QString::fromStdString(bind.action);
            //case 2: return QString::number(bind.lineNumber);
            case 0: return QString::fromStdString(join(bind.modifiers, ", "));
            case 3: return QString::fromStdString(join(bind.tags, ", "));
        }
    }
    return QVariant();
}

QVariant BindModel::headerData(int section, Qt::Orientation orientation, int role) const {
    if (role == Qt::DisplayRole && orientation == Qt::Horizontal) {
        switch (section) {
            case 1: return "Tecla";
            case 2: return "Acción";
            //case 2: return "Línea";
            case 0: return "Modificadores";
            case 3: return "Tags";
        }
    }
    return QVariant();
}
