//! Core shape representation and inference.

use serde_json::Value;
use std::collections::BTreeMap;

/// A Shape describes the inferred structure of a set of JSON samples.
///
/// A shape is a union: it may allow multiple scalar types, an object form,
/// an array form, and/or null. This is a lattice — merging two shapes
/// produces a shape that accepts everything either side accepted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shape {
    /// Whether `null` has been observed.
    pub nullable: bool,
    /// Scalar variants observed.
    pub scalars: ScalarSet,
    /// Object variant, if any object sample was observed.
    pub object: Option<ObjectShape>,
    /// Array element shape, if any array sample was observed.
    pub array: Option<Box<Shape>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScalarSet {
    pub boolean: bool,
    pub integer: bool,
    pub float: bool,
    pub string: bool,
}

impl ScalarSet {
    pub fn is_empty(&self) -> bool {
        !(self.boolean || self.integer || self.float || self.string)
    }
    pub fn merge(&mut self, other: &ScalarSet) {
        self.boolean |= other.boolean;
        self.integer |= other.integer;
        self.float |= other.float;
        self.string |= other.string;
    }
}

/// Object shape — field set with presence counts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectShape {
    /// Total number of object observations that contributed to this shape.
    pub total: u64,
    /// Fields in insertion order (first observation wins).
    pub fields: Vec<(String, Field)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub shape: Shape,
    /// How many object observations contained this field.
    pub count: u64,
}

impl ObjectShape {
    fn get_mut(&mut self, key: &str) -> Option<&mut Field> {
        self.fields
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, f)| f)
    }
    fn insert(&mut self, key: String, field: Field) {
        self.fields.push((key, field));
    }
    pub fn get(&self, key: &str) -> Option<&Field> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, f)| f)
    }
}

impl Shape {
    /// Build a shape from a single JSON value.
    pub fn from_value(v: &Value) -> Shape {
        let mut s = Shape::default();
        s.absorb(v);
        s
    }

    /// Fold a JSON value into this shape as one additional observation.
    pub fn absorb(&mut self, v: &Value) {
        match v {
            Value::Null => self.nullable = true,
            Value::Bool(_) => self.scalars.boolean = true,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    self.scalars.integer = true;
                } else {
                    self.scalars.float = true;
                }
            }
            Value::String(_) => self.scalars.string = true,
            Value::Array(items) => {
                let inner = self.array.get_or_insert_with(|| Box::new(Shape::default()));
                for item in items {
                    inner.absorb(item);
                }
            }
            Value::Object(map) => {
                let obj = self.object.get_or_insert_with(ObjectShape::default);
                obj.total += 1;
                // Track seen keys (to avoid double counting if duplicates).
                let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
                for (k, val) in map {
                    if seen.insert(k.as_str(), ()).is_some() {
                        continue;
                    }
                    if let Some(field) = obj.get_mut(k) {
                        field.count += 1;
                        field.shape.absorb(val);
                    } else {
                        obj.insert(
                            k.clone(),
                            Field {
                                shape: Shape::from_value(val),
                                count: 1,
                            },
                        );
                    }
                }
            }
        }
    }

    /// Merge another shape into this one (lattice union).
    pub fn merge(&mut self, other: &Shape) {
        self.nullable |= other.nullable;
        self.scalars.merge(&other.scalars);
        match (&mut self.object, &other.object) {
            (Some(a), Some(b)) => {
                a.total += b.total;
                for (k, bf) in &b.fields {
                    if let Some(af) = a.get_mut(k) {
                        af.count += bf.count;
                        af.shape.merge(&bf.shape);
                    } else {
                        a.insert(k.clone(), bf.clone());
                    }
                }
            }
            (None, Some(b)) => self.object = Some(b.clone()),
            _ => {}
        }
        match (&mut self.array, &other.array) {
            (Some(a), Some(b)) => a.merge(b),
            (None, Some(b)) => self.array = Some(b.clone()),
            _ => {}
        }
    }

    /// How many distinct top-level variants this shape admits.
    /// Null is counted separately.
    pub fn variant_count(&self) -> usize {
        let mut n = 0;
        if self.scalars.boolean {
            n += 1;
        }
        if self.scalars.integer {
            n += 1;
        }
        if self.scalars.float {
            n += 1;
        }
        if self.scalars.string {
            n += 1;
        }
        if self.object.is_some() {
            n += 1;
        }
        if self.array.is_some() {
            n += 1;
        }
        n
    }

    /// A shape with no positive observations (only `null`, or empty).
    pub fn is_empty(&self) -> bool {
        self.scalars.is_empty() && self.object.is_none() && self.array.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scalar_inference() {
        let s = Shape::from_value(&json!("hi"));
        assert!(s.scalars.string);
        assert!(!s.nullable);
    }

    #[test]
    fn int_vs_float() {
        let s = Shape::from_value(&json!(1));
        assert!(s.scalars.integer);
        assert!(!s.scalars.float);
        let s = Shape::from_value(&json!(1.5));
        assert!(!s.scalars.integer);
        assert!(s.scalars.float);
    }

    #[test]
    fn object_fields_tracked() {
        let s = Shape::from_value(&json!({"a": 1, "b": "x"}));
        let o = s.object.as_ref().unwrap();
        assert_eq!(o.total, 1);
        assert_eq!(o.fields.len(), 2);
        assert!(o.get("a").unwrap().shape.scalars.integer);
    }

    #[test]
    fn merging_makes_field_optional() {
        let mut s = Shape::from_value(&json!({"a": 1, "b": 2}));
        s.absorb(&json!({"a": 3}));
        let o = s.object.as_ref().unwrap();
        assert_eq!(o.total, 2);
        assert_eq!(o.get("a").unwrap().count, 2);
        assert_eq!(o.get("b").unwrap().count, 1);
    }

    #[test]
    fn merging_produces_union() {
        let mut s = Shape::from_value(&json!(1));
        s.absorb(&json!("hi"));
        assert!(s.scalars.integer && s.scalars.string);
    }

    #[test]
    fn null_makes_nullable() {
        let mut s = Shape::from_value(&json!({"a": 1}));
        s.absorb(&json!({"a": null}));
        let a = s.object.as_ref().unwrap().get("a").unwrap();
        assert!(a.shape.nullable);
        assert!(a.shape.scalars.integer);
    }

    #[test]
    fn array_elements_merged() {
        let s = Shape::from_value(&json!([1, 2, "three"]));
        let inner = s.array.as_ref().unwrap();
        assert!(inner.scalars.integer);
        assert!(inner.scalars.string);
    }
}
