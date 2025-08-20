use std::iter;
use core_relations::{CounterId, Database, DisplacedTable, TableId};

struct NetlistDatabase {
    db: Database,
    id_counter: CounterId,
    ts_counter: CounterId,
    displaced: TableId
}

impl NetlistDatabase {
    fn default() -> Self {
        let mut db = Database::default();
        let id_counter = db.add_counter();  // shared by both base and container values
        let ts_counter = db.add_counter();  // used for timestamps
        let displaced = db.add_table(DisplacedTable::default(), iter::empty(), iter::empty()); // union-find structure

        Self{db, id_counter, ts_counter, displaced}
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