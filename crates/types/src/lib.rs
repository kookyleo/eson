use std::collections::HashMap;

use parser::{JsonValue, Key};

#[derive(Debug)]
pub struct JsonInt(i64);

#[derive(Debug)]
pub struct JsonFloat(f64);

#[derive(Debug)]
pub struct JsonString(String);

#[derive(Debug)]
pub struct JsonBool(bool);

#[derive(Debug)]
pub struct JsonNull;

#[derive(Debug)]
pub struct JsonArray(Vec<JsonValue>);

#[derive(Debug)]
pub struct JsonObject(HashMap<Key, JsonValue>);

impl From<JsonInt> for JsonValue {
    fn from(i: JsonInt) -> JsonValue {
        JsonValue::Int(i.0)
    }
}

impl From<JsonFloat> for JsonValue {
    fn from(f: JsonFloat) -> JsonValue {
        JsonValue::Float(f.0)
    }
}

impl From<JsonInt> for i64 {
    fn from(i: JsonInt) -> i64 {
        i.0
    }
}

impl From<JsonFloat> for f64 {
    fn from(f: JsonFloat) -> f64 {
        f.0
    }
}

impl From<JsonString> for JsonValue {
    fn from(s: JsonString) -> JsonValue {
        JsonValue::Str(s.0)
    }
}

impl From<JsonBool> for JsonValue {
    fn from(b: JsonBool) -> JsonValue {
        JsonValue::Boolean(b.0)
    }
}

impl From<JsonNull> for JsonValue {
    fn from(_: JsonNull) -> JsonValue {
        JsonValue::Null
    }
}

impl From<JsonArray> for JsonValue {
    fn from(a: JsonArray) -> JsonValue {
        JsonValue::Array(a.0)
    }
}

impl From<JsonObject> for JsonValue {
    fn from(o: JsonObject) -> JsonValue {
        JsonValue::Object(o.0)
    }
}

impl From<JsonString> for String {
    fn from(s: JsonString) -> String {
        s.0
    }
}

impl From<JsonBool> for bool {
    fn from(b: JsonBool) -> bool {
        b.0
    }
}

impl From<JsonNull> for () {
    fn from(_: JsonNull) -> () {
        ()
    }
}

impl From<JsonArray> for Vec<JsonValue> {
    fn from(a: JsonArray) -> Vec<JsonValue> {
        a.0
    }
}

impl From<JsonObject> for HashMap<Key, JsonValue> {
    fn from(o: JsonObject) -> HashMap<Key, JsonValue> {
        o.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn reg() {
        // type DynFn = Box<dyn Fn(Vec<JsonValue>) -> JsonValue>;
        // let mut functions: HashMap<String, DynFn> = HashMap::new();
        //
        // fn test_add(args: Vec<JsonInt>) -> JsonInt {
        //     let mut sum = 0;
        //     for arg in args {
        //         sum += <JsonInt as Into<i64>>::into(arg);
        //     }
        //     JsonInt(sum)
        // }
        //
        // functions.insert("add".to_string(), Box::new(test_add));
        //

        // functions.insert("add".to_string(), Box::new(|args| {
        //     let mut sum = 0.0;
        //     for arg in args {
        //         sum += <JsonValue as Into<f64>>::into(arg);
        //     }
        //     JsonValue::Float(sum)
        // }));

        // let args = vec![JsonValue::Float(1.0), JsonValue::Float(2.0), JsonValue::Float(3.0)];
        // let result = functions["add"](args);
        // assert_eq!(result, JsonValue::Float(6.0));
    }

    #[test]
    fn test_object() {
        let mut o = JsonObject(HashMap::new());
        o.0.insert(Key::from("hello"), JsonValue::Int(1));
        o.0.insert(Key::from("world"), JsonValue::Int(2));
        let oo: HashMap<Key, JsonValue> = o.into();
        assert_eq!(oo, {
            let mut m = HashMap::new();
            m.insert(Key::from("hello"), JsonValue::Int(1));
            m.insert(Key::from("world"), JsonValue::Int(2));
            m
        });

        let mut o = JsonObject(HashMap::new());
        o.0.insert(Key::from("hello"), JsonValue::Int(1));
        o.0.insert(Key::from("world"), JsonValue::Int(2));
        let oo: JsonValue = o.into();
        assert_eq!(
            oo,
            JsonValue::Object({
                let mut m = HashMap::new();
                m.insert(Key::from("hello"), JsonValue::Int(1));
                m.insert(Key::from("world"), JsonValue::Int(2));
                m
            })
        );

        let mut o = JsonObject(HashMap::new());
        o.0.insert(Key::from("hello"), JsonValue::Int(1));
        o.0.insert(Key::from("world"), JsonValue::Int(2));
        let oo = JsonValue::Object({
            let mut m = HashMap::new();
            m.insert(Key::from("hello"), JsonValue::Int(1));
            m.insert(Key::from("world"), JsonValue::Int(2));
            m
        });
        assert_eq!(oo, o.into());
    }

    #[test]
    fn test_array() {
        let a = JsonArray(vec![
            JsonValue::Int(1),
            JsonValue::Int(2),
            JsonValue::Int(3),
        ]);
        let aa: Vec<JsonValue> = a.into();
        assert_eq!(
            aa,
            vec![JsonValue::Int(1), JsonValue::Int(2), JsonValue::Int(3)]
        );

        let a = JsonArray(vec![
            JsonValue::Int(1),
            JsonValue::Int(2),
            JsonValue::Int(3),
        ]);
        let aa: JsonValue = a.into();
        assert_eq!(
            aa,
            JsonValue::Array(vec![
                JsonValue::Int(1),
                JsonValue::Int(2),
                JsonValue::Int(3),
            ])
        );

        let a = JsonArray(vec![
            JsonValue::Int(1),
            JsonValue::Int(2),
            JsonValue::Int(3),
        ]);
        let aa = JsonValue::Array(vec![
            JsonValue::Int(1),
            JsonValue::Int(2),
            JsonValue::Int(3),
        ]);
        assert_eq!(aa, a.into());
    }

    #[test]
    fn test_null() {
        let n = JsonNull;
        let nn: JsonValue = n.into();
        assert_eq!(nn, JsonValue::Null);

        let n = JsonNull;
        let nn: () = n.into();
        assert_eq!(nn, ());

        let n = JsonNull;
        let nn = ();
        assert_eq!(nn, n.into());
    }

    #[test]
    fn test_string() {
        let s = JsonString("hello".to_string());
        let ss: String = s.into();
        assert_eq!(ss, "hello");

        let s = JsonString("hello".to_string());
        let ss: JsonValue = s.into();
        assert_eq!(ss, JsonValue::Str("hello".to_string()));

        let s = JsonString("hello".to_string());
        let ss = JsonValue::Str("hello".to_string());
        assert_eq!(ss, s.into());
    }

    #[test]
    fn test_bool() {
        let b = JsonBool(true);
        let bb: bool = b.into();
        assert_eq!(bb, true);

        let b = JsonBool(true);
        let bb: JsonValue = b.into();
        assert_eq!(bb, JsonValue::Boolean(true));

        let b = JsonBool(true);
        let bb = JsonValue::Boolean(true);
        assert_eq!(bb, b.into());
    }

    #[test]
    fn test_float() {
        let f = JsonFloat(42.0);
        let ff: f64 = f.into();
        assert_eq!(ff, 42.0);

        let f = JsonFloat(42.0);
        let ff: JsonValue = f.into();
        assert_eq!(ff, JsonValue::Float(42.0));

        let f = JsonFloat(42.0);
        let ff = JsonValue::Float(42.0);
        assert_eq!(ff, f.into());
    }

    fn test_add(a: i64, b: i64) -> i64 {
        a + b
    }

    #[test]
    fn test2() {
        let a = JsonInt(1);
        let b = JsonInt(2);
        let c = test_add(a.into(), b.into());
        assert_eq!(c, 3);
    }

    #[test]
    fn test() {
        let i = JsonInt(42);
        let f = JsonFloat(42.0);
        let ji: JsonValue = i.into();
        let jf: JsonValue = f.into();
        assert_eq!(ji, JsonValue::Int(42));
        assert_eq!(jf, JsonValue::Float(42.0));

        let i = JsonInt(42);
        let f = JsonFloat(42.0);
        let ii: i64 = i.into();
        let ff: f64 = f.into();
        assert_eq!(ii, 42);
        assert_eq!(ff, 42.0);

        let i = JsonValue::Int(42);
        let f = JsonValue::Float(42.0);
        assert_eq!(ii, 42);
        assert_eq!(ff, 42.0);
    }
}
