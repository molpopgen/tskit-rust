fn make_tables() -> tskit::TableCollection {
    let mut tables = tskit::TableCollection::new(100.).unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 0.0, -1, -1)
        .unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 1.0, -1, -1)
        .unwrap();
    tables
        .add_node(tskit::NodeFlags::default(), 1.0, -1, -1)
        .unwrap();
    tables.add_edge(0., 50., 1, 0).unwrap();
    tables.add_edge(0., 50., 2, 0).unwrap();
    tables
}

fn get_edges_from_tables(tables: &tskit::TableCollection) -> Vec<tskit::EdgeTableRow> {
    tables.edges().iter().collect::<Vec<_>>()
}

fn get_edges_via_table_iteration_trait<T>(tables: &T) -> Vec<tskit::EdgeTableRow>
where
    T: tskit::TableIteration,
{
    tables.edges().iter().collect::<Vec<_>>()
}

fn get_edges_via_table_iteration_trait_object(
    tables: &dyn tskit::ObjectSafeTableIteration,
) -> Vec<tskit::EdgeTableRow> {
    tskit::ObjectSafeTableIteration::edges_iter(tables).collect::<Vec<_>>()
}

#[test]
fn test_table_collection_edge_iteration() {
    let tables = make_tables();
    let v0 = get_edges_from_tables(&tables);
    let v1 = get_edges_via_table_iteration_trait(&tables);
    assert_eq!(v0, v1);
}
#[test]
fn test_table_collection_edge_iteration_object_safety() {
    let tables = Box::new(make_tables());
    let v0 = get_edges_from_tables(&tables);
    let v1 = get_edges_via_table_iteration_trait_object(&tables);
    assert_eq!(v0, v1);
}
