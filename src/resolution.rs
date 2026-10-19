use std::{
    ops::Deref,
    process::Output,
    rc::{self, Rc},
};

use crate::{
    DataTable, FeatureInstance, FeatureSchema, FeatureSymbol, PartInstance, PartSchema, PartSymbol,
    Schema, StructureInstance, StructureSchema, StructureSymbol, structure,
};

/*
 * ============
 * FEATURE DATA
 * ============
 */

#[derive(Clone, Copy, Debug)]
pub enum Presence {
    Required,
    Optional,
}

#[derive(Clone, Debug)]
pub enum Relation<Symbol: Clone> {
    MustSelfDefine,
    PrefersSelfDefined(ReferencePath<Symbol>),
    MustInherit(ReferencePath<Symbol>),
    PrefersInherited(ReferencePath<Symbol>),
}

#[derive(Clone, Debug)]
pub struct ReferencePath<Symbol: Clone>(pub Vec<Reference<Symbol>>);

#[derive(Clone, Debug)]
pub enum Reference<Symbol: Clone> {
    Parent,
    Sibling(PartSymbol),
    Child(PartSymbol),

    /// This is only used by Parts to pull
    /// data from their embedded Structure.
    ///
    /// Using this inside a Structure prohibited;
    /// any resolution of such use will return a
    /// `ResolutionError`.
    SubStructure(Symbol),
    Finally(Symbol),
}

/*
 * ==========
 * RESOLUTION
 * ==========
 */

pub struct Sourced<'a, Root: DataSource, Symbol: Clone> {
    pub root: &'a Root,
    pub path: ReferencePath<Symbol>,
}

pub enum StopOrContinue<T> {
    Stop(T),
    Continue(T),
}

impl<T> StopOrContinue<T> {
    pub const fn value(&self) -> &T {
        match self {
            StopOrContinue::Stop(v) | StopOrContinue::Continue(v) => v,
        }
    }
}

impl<Symbol: Clone> ReferencePath<Symbol> {}

impl<'a, Root: DataSource, Symbol: Clone> Sourced<'a, Root, Symbol> {}

pub struct Defined<Source: DataSource, Object> {
    pub data_source: Source,
    pub instance: Object,
}

pub struct Referenced<Origin: ReferenceOrigin<Symbol>, Symbol: Clone> {
    pub reference_point: Origin,
    pub instance: Symbol,
}

pub trait ReferenceOrigin<Symbol: Clone> {
    type Output;

    fn resolve_absolute_reference(&self, reference_path: ReferencePath<Symbol>) -> Self::Output;
}

impl<Source: DataSource> ReferenceOrigin<PartSymbol> for Defined<Source, StructureSchema> {
    type Output = PartSchema;

    fn resolve_absolute_reference(
        &self,
        reference_path: ReferencePath<PartSymbol>,
    ) -> Self::Output {
        todo!()
    }
}

pub trait DataSource {
    type Feature;
    type Part;
    type Structure;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature>;
    fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part>;
    fn get_structure(&self, structure: &StructureSymbol) -> Option<&Self::Structure>;
}

// impl<T: DataSource, Instance> DataSource for Defined<T, Instance> {
//     type Feature = T::Feature;
//     type Part = T::Part;
//
//     fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
//         todo!()
//     }
//
//     fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part> {
//         todo!()
//     }
// }

impl DataSource for DataTable {
    type Feature = FeatureSchema;
    type Part = PartSchema;
    type Structure = StructureSchema;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
        match self.get_schema(*feature).map(|schema| match schema {
            Schema::Feature(schema) => Some(schema),
            _ => None,
        }) {
            Some(opt) => opt,
            None => None,
        }
    }

    fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part> {
        match self.get_schema(*part).map(|schema| match schema {
            Schema::Part(schema) => Some(schema),
            _ => None,
        }) {
            Some(opt) => opt,
            None => None,
        }
    }

    fn get_structure(&self, structure: &StructureSymbol) -> Option<&Self::Structure> {
        match self.get_schema(*structure).map(|schema| match schema {
            Schema::Structure(schema) => Some(schema),
            _ => None,
        }) {
            Some(opt) => opt,
            None => None,
        }
    }
}

// impl DataSource for Defined<DataTable, StructureInstance> {
//     type Feature = FeatureInstance;
//     type Part = PartInstance;
//
//     fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
//         match self
//             .data_source
//             .get_structure(&self.instance.schema)?
//             .get_feature(feature)?
//         {
//             Relation::MustSelfDefine => self.get_feature(feature),
//             Relation::PrefersSelfDefined(reference) => {
//                 if let Some(f) = self.get_feature(feature) {
//                     Some(f)
//                 } else {
//                     // uhhhhhhhhhh uhhhhhh
//                     None
//                 }
//             }
//             Relation::MustInherit(reference) => {
//                 // uhhhhhhhhhh uhhhhhh
//                 None
//             }
//             Relation::PrefersInherited(reference) => {
//                 // uhhhhhhhhhh uhhhhhh
//                 None
//             }
//         }
//     }
//
//     fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part> {
//         todo!()
//     }
// }

pub struct ValidatedInstance<'a>(&'a StructureInstance);

impl<'a> ValidatedInstance<'a> {
    pub fn structure(&self) -> &'a StructureInstance {
        self.0
    }
}

pub enum ResolutionError {
    ImpossibleParent,
    PartNotFound,
    FeatureNotFound,
    NoTableEntry,
    IllegalStructureReference,
    Placeholder,
}

pub enum ValiditationError {
    Resolution(ResolutionError),
    NoInformation,
}

impl StructureInstance {
    pub fn validate<'a>(
        structure: &'a StructureInstance,
    ) -> Result<ValidatedInstance<'a>, ValiditationError> {
        Ok(ValidatedInstance(structure))
    }
}
