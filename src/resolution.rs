use std::rc::Rc;

use crate::{
    DataTable, FeatureInstance, FeatureSymbol, PartInstance, PartSchema, PartSymbol,
    StructureInstance, StructureSchema,
};

/*
 * ============
 * FEATURE DATA
 * ============
 */

#[derive(Clone, Copy)]
pub enum Presence {
    Required,
    Optional,
}

#[derive(Clone)]
pub enum Relation<Symbol: Clone> {
    MustSelfDefine,
    PrefersSelfDefined(Reference<Symbol>),
    MustInherit(Reference<Symbol>),
    PrefersInherited(Reference<Symbol>),
}

#[derive(Clone)]
pub enum Reference<Symbol: Clone> {
    Parent(Option<Rc<Reference<Symbol>>>),
    Sibling(PartSymbol, Option<Rc<Reference<Symbol>>>),
    Child(PartSymbol, Option<Rc<Reference<Symbol>>>),

    /// This is only used by Parts to pull
    /// data from their embedded Structure.
    ///
    /// Using this inside a Structure prohibited;
    /// any resolution of such use will return a
    /// `ResolutionError`.
    SubStructure(Symbol, Option<Rc<Reference<Symbol>>>),
    Finally(Symbol),
}

/*
 * ==========
 * RESOLUTION
 * ==========
 */

pub struct Sourced<'a, Root: Structure, Symbol: Clone> {
    pub root: &'a Root,
    pub path: Reference<Symbol>,
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

impl<Symbol: Clone> Reference<Symbol> {
    pub fn accumulate_until<'a, Acc: Clone, F>(&'a self, acc: Acc, mut f: F) -> Acc
    where
        F: FnMut(&Acc, &'a Reference<Symbol>) -> StopOrContinue<Acc>,
    {
        match self {
            Reference::Parent(reference)
            | Reference::Sibling(_, reference)
            | Reference::Child(_, reference)
            | Reference::SubStructure(_, reference) => match f(&acc, self) {
                StopOrContinue::Stop(n_acc) => n_acc,
                StopOrContinue::Continue(n_acc) => match reference {
                    Some(reference) => reference.accumulate_until(n_acc, f),
                    None => n_acc,
                },
            },

            Reference::Finally(_) => f(&acc, self).value().clone(),
        }
    }

    pub fn accumulate<Acc: Copy, F>(&self, acc: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, &Reference<Symbol>) -> Acc,
    {
        self.accumulate_until(acc, |a, r| StopOrContinue::Continue(f(*a, r)))
    }

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

    pub fn append<'b>(&'b self, sub_reference: Reference<Symbol>) -> Option<Reference<Symbol>> {
        let last = self.last();

        match last {
            Reference::Parent(_) => Some(Reference::Parent(Some(Rc::new(sub_reference)))),
            Reference::Sibling(symbol, _) => Some(Reference::Sibling(
                symbol.clone(),
                Some(Rc::new(sub_reference)),
            )),
            Reference::Child(symbol, _) => Some(Reference::Child(
                symbol.clone(),
                Some(Rc::new(sub_reference)),
            )),
            Reference::SubStructure(symbol, _) => Some(Reference::SubStructure(
                symbol.clone(),
                Some(Rc::new(sub_reference)),
            )),
            Reference::Finally(_) => None,
        }
    }
}

impl<'a, Root: Structure, Symbol: Clone> Sourced<'a, Root, Symbol> {}

pub struct Defined<Instance> {
    pub data_table: DataTable,
    pub instance: Instance,
}

pub trait Structure {
    type Feature;
    type Part;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature>;
    fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part>;
}

impl Structure for StructureSchema {
    type Feature = Relation<FeatureSymbol>;
    type Part = PartSchema;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
        self.features.get(&feature)
    }

    fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part> {
        self.parts.get(part)
    }
}

impl Structure for Defined<StructureInstance> {
    type Feature = FeatureInstance;
    type Part = PartInstance;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
        match self
            .data_table
            .get_structure(&self.instance.schema)?
            .get_feature(feature)?
        {
            Relation::MustSelfDefine => self.get_feature(feature),
            Relation::PrefersSelfDefined(reference) => {
                if let Some(f) = self.get_feature(feature) {
                    Some(f)
                } else {
                    // uhhhhhhhhhh uhhhhhh
                    None
                }
            }
            Relation::MustInherit(reference) => {
                // uhhhhhhhhhh uhhhhhh
                None
            }
            Relation::PrefersInherited(reference) => {
                // uhhhhhhhhhh uhhhhhh
                None
            }
        }
    }

    fn get_part(&self, part: &PartSymbol) -> Option<&Self::Part> {
        todo!()
    }
}

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
