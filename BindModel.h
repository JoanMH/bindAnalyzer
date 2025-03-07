#ifndef BINDMODEL_H
#define BINDMODEL_H

#include <QAbstractTableModel>
#include "Bind.h"

class BindModel : public QAbstractTableModel {
    Q_OBJECT

private:
    std::vector<Bind> binds;

public:
    explicit BindModel(QObject* parent = nullptr);

    void setBinds(const std::vector<Bind>& newBinds);

    int rowCount(const QModelIndex& parent = QModelIndex()) const override;
    int columnCount(const QModelIndex& parent = QModelIndex()) const override;
    QVariant data(const QModelIndex& index, int role = Qt::DisplayRole) const override;
    QVariant headerData(int section, Qt::Orientation orientation, int role = Qt::DisplayRole) const override;
};

#endif // BINDMODEL_H
