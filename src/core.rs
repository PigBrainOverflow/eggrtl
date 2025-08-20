use core_relations::{CounterId, Database, DisplacedTable, TableId, Value, Rebuilder, ContainerValue, SortedWritesTable, ColumnId};
use numeric_id::NumericId;
use std::{collections::HashMap, iter};
use bimap::BiHashMap;

struct NetlistDatabase {
    db: Database,
    id_counter: CounterId,
    ts_counter: CounterId,
    types: BiHashMap<&'static str, Value>,  // we use external containers to record types and wires
    wires: BiHashMap<i64, Value>,
    displaced: TableId,
    ay_cells: TableId,
    aby_cells: TableId,

}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct VecContainer(Vec<Value>);
impl ContainerValue for VecContainer {
    fn rebuild_contents(&mut self, rebuilder: &dyn Rebuilder) -> bool {
        rebuilder.rebuild_slice(&mut self.0)
    }

    fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.0.iter().copied()
    }
}

impl NetlistDatabase {
    fn default() -> Self {
        let mut db = Database::default();
        let id_counter = db.add_counter();  // shared by both base and container values
        let ts_counter = db.add_counter();  // used for timestamps
        let displaced = db.add_table(DisplacedTable::default(), iter::empty(), iter::empty()); // union-find structure

        db.base_values_mut().register_type::<i64>();    // register i64
        db.base_values_mut().register_type::<&'static str>(); // register str
        db.container_values_mut().register_type::<VecContainer>(id_counter, move |state, old, new| {
            if old != new {
                let next_ts = Value::from_usize(state.read_counter(ts_counter));
                state.stage_insert(displaced, &[old, new, next_ts]);
                std::cmp::min(old, new)
            }
            else {
                old
            }
        });

        // (type, a, y, t)
        let ay_cells_impl = SortedWritesTable::new(
            2, 4, Some(ColumnId::new(3)), Vec::new(),
            Box::new(move |state, expr1, expr2, res| {
                if expr1 != expr2 {
                    let vec1 = &state.container_values().get_val::<VecContainer>(expr1[2]).unwrap().0;
                    let vec2 = &state.container_values().get_val::<VecContainer>(expr2[2]).unwrap().0;
                    assert_eq!(vec1.len(), vec2.len());
                    for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {    // union each pair of elements
                        if elem1 != elem2 {
                            state.stage_insert(displaced, &[*elem1, *elem2, expr2[3]]);
                        }
                    }
                    res.extend_from_slice(expr2);   // expr2 wins
                    true
                }
                else {
                    false
                }
            })
        );
        let ay_cells = db.add_table(ay_cells_impl, iter::once(displaced), iter::once(displaced));

        // (type, a, b, y, t)
        let aby_cells_impl = SortedWritesTable::new(
            3, 5, Some(ColumnId::new(4)), Vec::new(),
            Box::new(move |state, expr1, expr2, res| {
                if expr1 != expr2 {
                    let vec1 = &state.container_values().get_val::<VecContainer>(expr1[3]).unwrap().0;
                    let vec2 = &state.container_values().get_val::<VecContainer>(expr2[3]).unwrap().0;
                    assert_eq!(vec1.len(), vec2.len());
                    for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {    // union each pair of elements
                        if elem1 != elem2 {
                            state.stage_insert(displaced, &[*elem1, *elem2, expr2[4]]);
                        }
                    }
                    // state.stage_insert(displaced, &[expr1[3], expr2[3], expr2[4]]); // union the vecs
                    res.extend_from_slice(expr2);   // expr2 wins
                    true
                }
                else {
                    false
                }
            })
        );
        let aby_cells = db.add_table(aby_cells_impl, iter::once(displaced), iter::once(displaced));

        let absy_cells_impl = SortedWritesTable::new(
            4, 6, Some(ColumnId::new(5)), Vec::new(),
            Box::new(move |state, expr1, expr2, res| {
                if expr1 != expr2 {
                    let vec1 = &state.container_values().get_val::<VecContainer>(expr1[4]).unwrap().0;
                    let vec2 = &state.container_values().get_val::<VecContainer>(expr2[4]).unwrap().0;
                    assert_eq!(vec1.len(), vec2.len());
                    for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {    // union each pair of elements
                        if elem1 != elem2 {
                            state.stage_insert(displaced, &[*elem1, *elem2, expr2[5]]);
                        }
                    }
                    res.extend_from_slice(expr2);   // expr2 wins
                    true
                }
                else {
                    false
                }
            })
        );
        let absy_cells = db.add_table(absy_cells_impl, iter::once(displaced), iter::once(displaced));

        let ay_types = vec!["$not", "$logic_not"];
        let aby_types = vec![
            "$and", "$or", "$xor", "$nand", "$nor", "$xnor",
            "$adds", "$addu", "$subs", "$subu", "$muls", "$mulu", "$divs", "$divu", "$mod"
        ];
        let absy_types = vec!["$mux"];
        let mut types = BiHashMap::new();
        for ty in ay_types.iter().chain(aby_types.iter()).chain(absy_types.iter()) {
            types.insert(*ty, Value::from_usize(db.inc_counter(id_counter)));
        }

        Self{db, id_counter, ts_counter, types, wires: BiHashMap::new(), displaced, ay_cells, aby_cells}
    }

    fn build_from_json(&mut self, json_path: &str, top_mod: &str) {
        let netlist: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(json_path).expect("Failed to open JSON file")
        ).expect("Failed to parse JSON file");
        let top_module = netlist.get("modules")
            .and_then(|m| m.get(top_mod))
            .expect("Top module not found");

        
    }
}