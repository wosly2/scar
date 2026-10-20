use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use derive_more::{From, TryInto};

use crate::{Presence, Relation};

/*
 * ==========
 * DATA TABLE
 * ==========
 */

pub struct DataTable {
    schemas: HashMap<Symbol, Schema>,
    next_id: usize,
}

pub enum ObjectType {
    Part,
    Structure,
    Feature,
}

pub trait ObjectTyped {
    fn object_type(&self) -> ObjectType;
}

impl ObjectTyped for Schema {
    fn object_type(&self) -> ObjectType {
        match self {
            Schema::Part(_) => ObjectType::Part,
            Schema::Structure(_) => ObjectType::Structure,
            Schema::Feature(_) => ObjectType::Feature,
        }
    }
}

impl ObjectTyped for Symbol {
    fn object_type(&self) -> ObjectType {
        match self {
            Symbol::Part(_) => ObjectType::Part,
            Symbol::Structure(_) => ObjectType::Structure,
            Symbol::Feature(_) => ObjectType::Feature,
        }
    }
}

#[derive(TryInto, From, Debug, Clone)]
pub enum Schema {
    Part(PartSchema),
    Structure(StructureSchema),
    Feature(FeatureSchema),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, TryInto, From, Debug)]
pub enum Symbol {
    Part(PartSymbol),
    Structure(StructureSymbol),
    Feature(FeatureSymbol),
}

impl DataTable {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn new_symbol(&mut self, object: ObjectType) -> Symbol {
        let id = self.next_id;
        self.next_id += 1;

        match object {
            ObjectType::Part => Symbol::Part(PartSymbol(id)),
            ObjectType::Structure => Symbol::Structure(StructureSymbol(id)),
            ObjectType::Feature => Symbol::Feature(FeatureSymbol(id)),
        }
    }

    pub fn define(&mut self, schema: impl Into<Schema>) -> Symbol {
        let schema = schema.into();

        let symbol = self.new_symbol(schema.object_type());
        self.schemas.insert(symbol, schema);
        symbol
    }

    pub fn get_schema(&self, id: impl Into<Symbol>) -> Option<&Schema> {
        self.schemas.get(&id.into())
    }
}

/*
 * =======
 * SYMBOLS
 * =======
 */

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct StructureSymbol(pub usize);

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct FeatureSymbol(pub usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct FeatureEnumSymbol(pub usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PartSymbol(pub usize);

/*
 * ======
 * SCHEMA
 * ======
 */

#[derive(Clone, Debug)]
pub struct StructureSchema {
    pub features: HashMap<FeatureSymbol, Relation>,
    pub parts: HashSet<PartSymbol>,
}

#[derive(Clone, Debug)]
pub struct PartSchema {
    pub features: HashMap<FeatureSymbol, Relation>,
    pub structure: StructureSymbol,
    pub presence: Presence,
}

#[derive(Debug, Clone)]
pub struct FeatureSchema {
    pub value_type: FeatureValueSchema,
}

#[derive(Debug, Clone)]
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
