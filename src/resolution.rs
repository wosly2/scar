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
pub enum ReferencePath<Symbol: Clone> {
    Parent(Option<Rc<ReferencePath<Symbol>>>),
    Sibling(PartSymbol, Option<Rc<ReferencePath<Symbol>>>),
    Child(PartSymbol, Option<Rc<ReferencePath<Symbol>>>),

    /// This is only used by Parts to pull
    /// data from their embedded Structure.
    ///
    /// Using this inside a Structure prohibited;
    /// any resolution of such use will return a
    /// `ResolutionError`.
    SubStructure(Symbol, Option<Rc<ReferencePath<Symbol>>>),
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

impl<Symbol: Clone> ReferencePath<Symbol> {
    pub fn inner_rc<'a>(&'a self) -> Option<&'a Rc<ReferencePath<Symbol>>> {
        match self {
            ReferencePath::Parent(reference)
            | ReferencePath::Sibling(_, reference)
            | ReferencePath::Child(_, reference)
            | ReferencePath::SubStructure(_, reference) => match reference {
                Some(rc_ref) => Some(rc_ref),
                None => todo!(),
            },
            ReferencePath::Finally(_) => return None,
        }
    }

    pub fn inner_ref<'a>(&'a self) -> Option<&'a ReferencePath<Symbol>> {
        match self.inner_rc() {
            Some(rc_ref) => Some(rc_ref.as_ref()),
            None => None,
        }
    }

    pub fn accumulate_until<'a, Acc: Clone, F>(&'a self, acc: Acc, mut f: F) -> Acc
    where
        F: FnMut(&Acc, &'a ReferencePath<Symbol>) -> StopOrContinue<Acc>,
    {
        match self {
            ReferencePath::Parent(reference)
            | ReferencePath::Sibling(_, reference)
            | ReferencePath::Child(_, reference)
            | ReferencePath::SubStructure(_, reference) => match f(&acc, self) {
                StopOrContinue::Stop(n_acc) => n_acc,
                StopOrContinue::Continue(n_acc) => match reference {
                    Some(reference) => reference.accumulate_until(n_acc, f),
                    None => n_acc,
                },
            },

            ReferencePath::Finally(_) => f(&acc, self).value().clone(),
        }
    }

    pub fn accumulate<Acc: Copy, F>(&self, acc: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &ReferencePath<Symbol>) -> Acc,
    {
        self.accumulate_until(acc, |a, r| StopOrContinue::Continue(f(*a, r)))
    }

    /// this is 0-indexed
    pub fn nth<'a>(&'a self, n: usize) -> Option<&'a Self> {
        self.accumulate_until((0, None), |acc, refr| {
            if acc.0 == n {
                StopOrContinue::Stop((acc.0, Some(refr)))
            } else {
                StopOrContinue::Continue((acc.0 + 1, None))
            }
        })
        .1
    }

    /// all `Option<Reference>` are gauranteed to be `None`
    pub fn last<'a>(&'a self) -> &'a Self {
        self.accumulate_until(self, |_, refr| StopOrContinue::Continue(refr))
    }

    pub fn recursive_clone<'a>(&'a self) -> &'a Self {
        match self {
            ReferencePath::Parent(reference_path) => ReferencePath::Parent(match reference_path {
                Some(rp) => ReferencePath::Parent(Some(Rc::new(rp.deref()))),
                None => self,
            }),
            ReferencePath::Sibling(part_symbol, reference_path) => todo!(),
            ReferencePath::Child(part_symbol, reference_path) => todo!(),
            ReferencePath::SubStructure(_, reference_path) => todo!(),
            ReferencePath::Finally(_) => self,
        }
    }

    pub fn append<'b>(
        &'b self,
        sub_reference: &'b ReferencePath<Symbol>,
    ) -> Option<&'b ReferencePath<Symbol>> {
        let last = self.last();

        let new = self.clone();
        new.into_iter().last().map(|last| last = sub_reference);

        // let new_last = match last {
        //     ReferencePath::Parent(_) => Some(ReferencePath::Parent(Some(Rc::new(sub_reference)))),
        //     ReferencePath::Sibling(symbol, _) => Some(ReferencePath::Sibling(
        //         symbol.clone(),
        //         Some(Rc::new(sub_reference)),
        //     )),
        //     ReferencePath::Child(symbol, _) => Some(ReferencePath::Child(
        //         symbol.clone(),
        //         Some(Rc::new(sub_reference)),
        //     )),
        //     ReferencePath::SubStructure(symbol, _) => Some(ReferencePath::SubStructure(
        //         symbol.clone(),
        //         Some(Rc::new(sub_reference)),
        //     )),
        //     ReferencePath::Finally(_) => None,
        // };
    }
}

pub struct ReferencePathIterator<'a, Symbol: Clone> {
    reference_stub: Option<&'a ReferencePath<Symbol>>,
}

impl<'a, Symbol: Clone> Iterator for ReferencePathIterator<'a, Symbol> {
    type Item = &'a ReferencePath<Symbol>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = match self.reference_stub {
            Some(rc_ref) => rc_ref.inner_ref(),
            None => return None,
        };
        self.reference_stub = inner;
        inner
    }
}

impl<'a, Symbol: Clone> IntoIterator for &'a ReferencePath<Symbol> {
    type Item = &'a ReferencePath<Symbol>;
    type IntoIter = ReferencePathIterator<'a, Symbol>;

    fn into_iter(self) -> Self::IntoIter {
        ReferencePathIterator {
            reference_stub: Some(self),
        }
    }
}

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
        reference_path.accumulate(None, |acc, ref_path| match ref_path {
            ReferencePath::Parent(reference_path) => todo!(),
            ReferencePath::Sibling(part_symbol, reference_path) => todo!(),
            ReferencePath::Child(part_symbol, reference_path) => todo!(),
            ReferencePath::SubStructure(_, reference_path) => todo!(),
            ReferencePath::Finally(_) => todo!(),
        });
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
