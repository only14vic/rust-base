use serde_json::Value;

pub trait MergeJson {
    fn merge_json(&mut self, b: Value);
}

impl MergeJson for Value {
    fn merge_json(&mut self, b: Value) {
        match (self, b) {
            (a @ &mut Value::Object(_), Value::Object(b)) => {
                let a = a.as_object_mut().unwrap();
                for (k, v) in b {
                    Self::merge_json(a.entry(k).or_insert(Value::Null), v);
                }
            },
            (a, b) => *a = b
        }
    }
}
