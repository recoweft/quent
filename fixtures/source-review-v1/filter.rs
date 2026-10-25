fn review(table: quent_core::Table, predicate: quent_core::Predicate) {
    quent_core::Draft::delete(table).filter(predicate);
}
