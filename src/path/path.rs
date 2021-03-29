use serde_json::{Value, Map};
use serde_json::json;
use serde_json::value::Value::Array;
use crate::path::structures::{JsonPath, JsonPathIndex};

pub(crate) trait Path<'a> {
    type Data;
    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data>;
}

type PathInstance<'a> = Box<dyn Path<'a, Data=Value> + 'a>;

fn process_path<'a>(json_path: &'a JsonPath, root: &'a Value) -> PathInstance<'a> {
    match json_path {
        JsonPath::Root => Box::new(RootPointer::new(root)),
        JsonPath::Field(key) => Box::new(ObjectField::new(key)),
        JsonPath::Path(chain) => Box::new(Chain::from(chain, root)),
        JsonPath::Index(key, index) => Box::new(Chain::from_index(Box::new(ObjectField::new(key)), process_path_index(index, root))),
        _ => Box::new(EmptyPath {})
    }
}

fn process_path_index<'a>(json_path_index: &'a JsonPathIndex, _root: &'a Value) -> PathInstance<'a> {
    match json_path_index {
        JsonPathIndex::Single(index) => Box::new(ArrayIndex::new(*index)),
        JsonPathIndex::Slice(s, e, step) => Box::new(ArraySlice::new(*s, *e, *step)),
        _ => Box::new(EmptyPath {})
    }
}


pub(crate) struct EmptyPath {}

impl<'a> Path<'a> for EmptyPath {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        vec![&data]
    }
}

pub(crate) struct RootPointer<'a, T> {
    root: &'a T
}

impl<'a, T> RootPointer<'a, T> {
    pub(crate) fn new(root: &'a T) -> RootPointer<'a, T> {
        RootPointer { root }
    }
}

impl<'a> Path<'a> for RootPointer<'a, Value> {
    type Data = Value;

    fn path(&self, _data: &'a Self::Data) -> Vec<&'a Self::Data> {
        vec![self.root]
    }
}

#[derive(Debug)]
pub(crate) struct ArraySlice {
    start_index: i32,
    end_index: i32,
    step: usize,
}

impl ArraySlice {
    pub(crate) fn new(start_index: i32,
                      end_index: i32,
                      step: usize, ) -> ArraySlice {
        ArraySlice { start_index, end_index, step }
    }

    fn end(&self, len: i32) -> Option<usize> {
        if self.end_index >= 0 {
            if self.end_index > len { None } else { Some(self.end_index as usize) }
        } else {
            if self.end_index < len * -1 { None } else { Some((len - (self.end_index * -1)) as usize) }
        }
    }

    fn start(&self, len: i32) -> Option<usize> {
        if self.start_index >= 0 {
            if self.start_index > len { None } else { Some(self.start_index as usize) }
        } else {
            if self.start_index < len * -1 { None } else { Some((len - self.start_index * -1) as usize) }
        }
    }

    fn process<'a, T>(&self, elements: &'a Vec<T>) -> Vec<&'a T> {
        let len = elements.len() as i32;
        let mut filtered_elems: Vec<&T> = vec![];
        match (self.start(len), self.end(len)) {
            (Some(start_idx), Some(end_idx)) => {
                for idx in (start_idx..end_idx).step_by(self.step) {
                    if let Some(v) = elements.get(idx) {
                        filtered_elems.push(v)
                    }
                }
                filtered_elems
            }
            _ => filtered_elems
        }
    }
}

impl<'a> Path<'a> for ArraySlice {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .map(|elems| self.process(elems))
            .unwrap_or(vec![])
    }
}

pub(crate) struct ArrayIndex {
    index: usize
}

impl ArrayIndex {
    pub(crate) fn new(index: usize) -> Self {
        ArrayIndex { index }
    }
}

impl<'a> Path<'a> for ArrayIndex {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_array()
            .and_then(|elems| elems.get(self.index))
            .map(|e| vec![e])
            .unwrap_or(vec![])
    }
}

pub(crate) struct ObjectField<'a> {
    key: &'a String,
}

impl<'a> ObjectField<'a> {
    pub(crate) fn new(key: &'a String) -> ObjectField<'a> {
        ObjectField { key }
    }
}

impl<'a> Path<'a> for ObjectField<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        data.as_object()
            .and_then(|fileds| fileds.get(self.key))
            .map(|e| vec![e])
            .unwrap_or(vec![])
    }
}

struct Chain<'a> {
    chain: Vec<PathInstance<'a>>,
}

impl<'a> Chain<'a> {
    pub fn new(chain: Vec<PathInstance<'a>>) -> Self {
        Chain { chain }
    }
    pub fn from_index(key: PathInstance<'a>, index: PathInstance<'a>) -> Self {
        Chain::new(vec![key, index])
    }
    pub fn from(chain: &'a Vec<&'a JsonPath>, root: &'a Value) -> Self {
        Chain::new(chain.iter().map(|p| process_path(p, root)).collect())
    }
}

impl<'a> Path<'a> for Chain<'a> {
    type Data = Value;

    fn path(&self, data: &'a Self::Data) -> Vec<&'a Self::Data> {
        self.chain.iter().fold(vec![data], |inter_res, path| {
            inter_res.iter().flat_map(|d| path.path(d)).collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::path::structures::{JsonPath, parse, JsonPathIndex};
    use crate::path::path::{ArraySlice, Path, ArrayIndex, ObjectField, RootPointer, process_path};
    use serde_json::Value;
    use serde_json::json;

    #[test]
    fn array_slice_end_start_test() {
        let array = vec![0, 1, 2, 3, 4, 5];
        let len = array.len() as i32;
        let mut slice = ArraySlice::new(0, 0, 0);

        assert_eq!(slice.start(len).unwrap(), 0);
        slice.start_index = 1;

        assert_eq!(slice.start(len).unwrap(), 1);

        slice.start_index = 2;
        assert_eq!(slice.start(len).unwrap(), 2);

        slice.start_index = 5;
        assert_eq!(slice.start(len).unwrap(), 5);

        slice.start_index = 7;
        assert_eq!(slice.start(len), None);

        slice.start_index = -1;
        assert_eq!(slice.start(len).unwrap(), 5);

        slice.start_index = -5;
        assert_eq!(slice.start(len).unwrap(), 1);

        slice.end_index = 0;
        assert_eq!(slice.end(len).unwrap(), 0);

        slice.end_index = 5;
        assert_eq!(slice.end(len).unwrap(), 5);

        slice.end_index = -1;
        assert_eq!(slice.end(len).unwrap(), 5);

        slice.end_index = -5;
        assert_eq!(slice.end(len).unwrap(), 1);
    }

    #[test]
    fn slice_test() {
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut slice = ArraySlice::new(0, 6, 2);
        assert_eq!(slice.path(&array), vec![&json!(0), &json!(2), &json!(4)]);

        slice.step = 3;
        assert_eq!(slice.path(&array), vec![&json!(0), &json!(3)]);

        slice.start_index = -1;
        slice.end_index = 1;

        assert!(slice.path(&array).is_empty());

        slice.start_index = -10;
        slice.end_index = 10;

        assert_eq!(slice.path(&array), vec![&json!(1), &json!(4), &json!(7)]);
    }

    #[test]
    fn index_test() {
        let array = parse(r#"[0,1,2,3,4,5,6,7,8,9,10]"#).unwrap();

        let mut index = ArrayIndex::new(0);

        assert_eq!(index.path(&array), vec![&json!(0)]);
        index.index = 10;
        assert_eq!(index.path(&array), vec![&json!(10)]);
        index.index = 100;
        assert!(index.path(&array).is_empty());
    }

    #[test]
    fn object_test() {
        let res_income = parse(r#"{"product": {"key":42}}"#).unwrap();

        let key = String::from("product");
        let mut field = ObjectField::new(&key);
        assert_eq!(field.path(&res_income), vec![&json!({"key":42})]);

        let key = String::from("fake");

        field.key = &key;
        assert!(field.path(&res_income).is_empty());
    }

    #[test]
    fn root_test() {
        let res_income = parse(r#"{"product": {"key":42}}"#).unwrap();

        let root = RootPointer::<Value>::new(&res_income);

        assert_eq!(root.path(&res_income), vec![&res_income])
    }

    #[test]
    fn path_instance_test() {
        let json = parse(r#"{"v": {"k":{"f":42,"array":[0,1,2,3,4,5]}}}"#).unwrap();

        let root = JsonPath::Root;
        let path_inst = process_path(&root, &json);
        assert_eq!(path_inst.path(&json), vec![&json]);

        let field1 = JsonPath::Field(String::from("v"));

        let path_inst = process_path(&field1, &json);
        let exp_json = parse(r#"{"k":{"f":42,"array":[0,1,2,3,4,5]}}"#).unwrap();
        assert_eq!(path_inst.path(&json), vec![&exp_json]);

        let field2 = JsonPath::Field(String::from("k"));
        let field3 = JsonPath::Field(String::from("f"));

        let chain = vec![&root, &field1, &field2, &field3];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);
        let exp_json = parse(r#"42"#).unwrap();
        assert_eq!(path_inst.path(&json), vec![&exp_json]);

        let index = JsonPath::Index(String::from("array"), JsonPathIndex::Single(3));
        let chain = vec![&root, &field1, &field2, &index];
        let chain = JsonPath::Path(&chain);

        let path_inst = process_path(&chain, &json);
        let exp_json = parse(r#"3"#).unwrap();
        assert_eq!(path_inst.path(&json), vec![&exp_json]);

        let index = JsonPath::Index(String::from("array"), JsonPathIndex::Slice(1, -1, 2));
        let chain = vec![&root, &field1, &field2, &index];
        let chain = JsonPath::Path(&chain);
        let path_inst = process_path(&chain, &json);
        let one = json!(1);
        let tree = json!(3);
        assert_eq!(path_inst.path(&json), vec![&one, &tree]);
    }
}