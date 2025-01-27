### Introduction to JsonPath

The library provides the basic functionality to find the set of the data according to the filtering query. The idea
comes from XPath for XML structures. The details can be found [there](https://goessner.net/articles/JsonPath/)
Therefore JsonPath is a query language for JSON, similar to XPath for XML. The JsonPath query is a set of assertions to
specify the JSON fields that need to be verified.

### Simple examples

Let's suppose we have a following json:

```json
  {
  "shop": {
    "orders": [
      {
        "id": 1,
        "active": true
      },
      {
        "id": 2
      },
      {
        "id": 3
      },
      {
        "id": 4,
        "active": true
      }
    ]
  }
}
 ```

And we pursue to find all orders id having the field 'active'. We can construct the jsonpath instance like
that  ```$.shop.orders[?(@.active)].id``` and get the result ``` [1,4] ```

### The jsonpath description

#### Operators

| Operator | Description | Where to use |
| --- | --- | --- |
| `$` | Pointer to the root of the json. | It is gently advising to start every jsonpath from the root. Also, inside the filters to point out that the path is starting from the root.
| `@` | Pointer to the current element inside the filter operations. | It is used inside the filter operations to iterate the collection.
| `*` or `[*]` | Wildcard. It brings to the list all objects and elements regardless their names. | It is analogue a flatmap operation.
| `<..>`| Descent operation. It brings to the list all objects, children of that objects and etc  | It is analogue a flatmap operation.
| `.<name>` or `.['<name>']` | the key pointing to the field of the object | It is used to obtain the specific field.
| `['<name>' (, '<name>')]` | the list of keys | the same usage as for a single key but for list
| `[<number>]` | the filter getting the element by its index. |
| `[<number> (, <number>)]` | the list if elements of array according to their indexes representing these numbers. |
| `[<start>:<end>:<step>]` | slice operator to get a list of element operating with their indexes. By default step = 1, start = 0, end = array len. The elements can be omitted ```[:]```
| `[?(<expression>)]` | the logical expression to filter elements in the list. | It is used with arrays preliminary.

#### Filter expressions

The expressions appear in the filter operator like that `[?(@.len > 0)]`. The expression in general consists of the
following elements:

- Left and right operands, that is ,in turn, can be a static value,representing as a primitive type like a number,
  string value `'value'`, array of them or another json path instance.
- Expression sign, denoting what action can be performed

| Expression sign  | Description | Where to use |
| --- | --- | --- |
| `==`| Equal | To compare numbers or string literals
| `!=`| Unequal| To compare numbers or string literals in opposite way to equals
| `<` | Less | To compare numbers
| `>` | Greater | To compare numbers
| `<=`| Less or equal | To compare numbers
| `>=`| Greater or equal | To compare numbers
| `~=`| Regular expression | To find the incoming right side in the left side.
| `in`| Find left element in the list of right elements. |
| `nin`| The same one as saying above but carrying the opposite sense. |
| `size`| The size of array on the left size should be corresponded to the number on the right side. |
| `noneOf`| The left size has no intersection with right |
| `anyOf` | The left size has at least one intersection with right |
| `subsetOf` | The left is a subset of the right side
|  | Exists operator. | The operator checks the existens of the field depicted on the left side like that `[?(@.key.isActive)]`

### Examples

Given the json

 ```json
 {
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century",
        "price": 8.95
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh",
        "title": "Sword of Honour",
        "price": 12.99
      },
      {
        "category": "fiction",
        "author": "Herman Melville",
        "title": "Moby Dick",
        "isbn": "0-553-21311-3",
        "price": 8.99
      },
      {
        "category": "fiction",
        "author": "J. R. R. Tolkien",
        "title": "The Lord of the Rings",
        "isbn": "0-395-19395-8",
        "price": 22.99
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95
    }
  },
  "expensive": 10
}
 ```

| JsonPath | Result |
 | :------- | :----- |
| `$.store.book[*].author`| The authors of all books     |
| `$..book[?(@.isbn)]`          | All books with an ISBN number         |
| `$.store.*`                  | All things, both books and bicycles  |
| `$..author`                   | All authors                         |
| `$.store..price`             | The price of everything         |
| `$..book[2]`                 | The third book                      |
| `$..book[-2]`                 | The second to last book            |
| `$..book[0,1]`               | The first two books               |
| `$..book[:2]`                | All books from index 0 (inclusive) until index 2 (exclusive) |
| `$..book[1:2]`                | All books from index 1 (inclusive) until index 2 (exclusive) |
| `$..book[-2:]`                | Last two books                   |
| `$..book[2:]`                | Book number two from tail          |
| `$.store.book[?(@.price < 10)]` | All books in store cheaper than 10  |
| `$..book[?(@.price <= $.expensive)]` | All books in store that are not "expensive"  |
| `$..book[?(@.author =~ /.*REES/i)]` | All books matching regex (ignore case)  |
| `$..*`                        | Give me every thing

### The library

The library intends to provide the basic functionality for ability to find the slices of data using the syntax, saying
above. The dependency can be found as following:
``` jsonpath-rust = 0.1.0 ```

The basic example is the following one:

```rust
use crate::{JsonPathFinder};
use serde_json::{json, Value};

fn main() {
    let finder = JsonPathFinder::from_str(r#"{"first":{"second":[{"active":1},{"passive":1}]}}"#, "$.first.second[?(@.active)]")?;
    let slice_of_data: Vec<&Value> = finder.find();
    assert_eq!(slice_of_data, vec![&json!({"active":1})]);
}
```

or even simpler:

```rust
 use crate::{JsonPathFinder};
use serde_json::{json, Value};

fn test(json: &str, path: &str, expected: Vec<&Value>) {
    match JsonPathFinder::from_str(json, path) {
        Ok(finder) => assert_eq!(finder.find(), expected),
        Err(e) => panic!("error while parsing json or jsonpath: {}", e)
    }
}
```

also it will work with the instances of [[Value]] as well.

```rust
  use serde_json::Value;
use crate::path::{json_path_instance, PathInstance};

fn test(json: Value, path: &str) {
    let path = parse_json_path(path).map_err(|e| e.to_string())?;
    JsonPathFinder::new(json, path)
}
 ```

also the trait can be used:

```rust

use serde_json::{json, Value};
use jsonpath_rust::JsonPathQuery;

fn test() {
    let json: Value = serde_json::from_str("{}").expect("to get json");
    let v = json.path("$..book[?(@.author size 10)].title").expect("the path is correct");
    assert_eq!(v, json!([]));
}
```

#### The structure

```rust
pub enum JsonPath {
    Root,
    // <- $
    Field(String),
    // <- field of the object 
    Chain(Vec<JsonPath>),
    // <- the whole jsonpath
    Descent(String),
    // <- '..'
    Index(JsonPathIndex),
    // <- the set of indexes represented by the next structure [[JsonPathIndex]]
    Current(Box<JsonPath>),
    // <- @
    Wildcard,
    // <- *
    Empty, // the structure to avoid inconsistency
}

pub enum JsonPathIndex {
    Single(usize),
    // <- [1]
    UnionIndex(Vec<f64>),
    // <- [1,2,3]
    UnionKeys(Vec<String>),
    // <- ['key_1','key_2']
    Slice(i32, i32, usize),
    // [0:10:1]
    Filter(Operand, FilterSign, Operand), // <- [?(operand sign operand)]
}

```


