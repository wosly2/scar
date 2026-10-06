use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
};

use crate::{Presence, Relation};

/*
 * ==========
 * DATA TABLE
 * ==========
 */

pub struct DataTable {
    structures: HashMap<StructureSymbol, StructureSchema>,
    next_structure_id: StructureSymbol,
    features: HashMap<FeatureSymbol, FeatureSchema>,
    next_feature_id: FeatureSymbol,
}

impl DataTable {
    pub fn new() -> Self {
        Self {
            structures: HashMap::new(),
            next_structure_id: StructureSymbol(1),
            features: HashMap::new(),
            next_feature_id: FeatureSymbol(1),
        }
    }

    pub fn define_structure(&mut self, schema: StructureSchema) -> StructureSymbol {
        let id = self.next_structure_id;
        self.structures.insert(id, schema);
        self.next_structure_id = StructureSymbol(self.next_structure_id.0 + 1);
        id
    }

    pub fn define_feature(&mut self, schema: FeatureSchema) -> FeatureSymbol {
        let id = self.next_feature_id;
        self.features.insert(id, schema);
        self.next_feature_id = FeatureSymbol(self.next_feature_id.0 + 1);
        id
    }

    pub fn get_structure(&self, id: &StructureSymbol) -> Option<&StructureSchema> {
        self.structures.get(&id)
    }

    pub fn get_feature(&self, id: &FeatureSymbol) -> Option<&FeatureSchema> {
        self.features.get(&id)
    }
}

/*
 * =======
 * SYMBOLS
 * =======
 */

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct StructureSymbol(pub usize);

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct FeatureSymbol(pub usize);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct FeatureEnumSymbol(pub String);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PartSymbol(pub String);

pub trait NewStringSymbol {
    fn new(s: impl Into<String>) -> Self;
}

impl NewStringSymbol for PartSymbol {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl NewStringSymbol for FeatureEnumSymbol {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/*
 * ======
 * SCHEMA
 * ======
 */

#[derive(Clone)]
pub struct StructureSchema {
    pub features: HashMap<FeatureSymbol, Relation<FeatureSymbol>>,
    pub parts: HashMap<PartSymbol, PartSchema>,
}

#[derive(Clone)]
pub struct PartSchema {
    pub features: HashMap<FeatureSymbol, Relation<FeatureSymbol>>,
    pub structure: StructureSymbol,
    pub presence: Presence,
}

pub struct FeatureSchema {
    pub value_type: FeatureValueSchema,
}

pub enum FeatureValueSchema {
    Type(TypeId),
    Enum(HashMap<FeatureEnumSymbol, Option<TypeId>>),
}

/* ========
 * INSTANCE
 * ========
 */

pub struct StructureInstance {
    pub schema: StructureSymbol,
    pub features: HashMap<FeatureSymbol, FeatureInstance>,
    pub parts: HashMap<PartSymbol, PartInstance>,
}

pub struct PartInstance {
    pub features: HashMap<FeatureSymbol, FeatureInstance>,
    pub structure: StructureInstance,
}

pub struct FeatureInstance {
    pub schema: FeatureSymbol,
    pub value: FeatureValueInstance,
}

pub enum FeatureValueInstance {
    Type(Rc<dyn Any>),
    Enum(FeatureEnumSymbol, Rc<dyn Any>),
}

pub trait KeyPair {
    type Key;
    fn key_pair(self) -> (Self::Key, Self);
}

impl KeyPair for StructureInstance {
    type Key = StructureSymbol;
    fn key_pair(self) -> (Self::Key, Self) {
        (self.schema, self)
    }
}

impl KeyPair for FeatureInstance {
    type Key = FeatureSymbol;
    fn key_pair(self) -> (Self::Key, Self) {
        (self.schema, self)
    }
}
