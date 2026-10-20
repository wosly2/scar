use std::{
    io::Error,
    ops::Deref,
    process::Output,
    rc::{self, Rc},
};

use crate::{
    DataTable, FeatureInstance, FeatureSchema, FeatureSymbol, PartInstance, PartSchema, PartSymbol,
    Schema, StructureInstance, StructureSchema, StructureSymbol, Symbol, structure,
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
pub enum Relation {
    MustSelfDefine,
    PrefersSelfDefined(ComplexReferencePath),
    MustInherit(ComplexReferencePath),
    PrefersInherited(ComplexReferencePath),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComplexReference {
    Parent,
    Sibling(PartSymbol),
    Child(PartSymbol),

    /// This is only used by Parts to pull
    /// data from their embedded Structure.
    ///
    /// Using this inside a Structure prohibited;
    /// any resolution of such use will return a
    /// `ResolutionError`.
    Substructure(FeatureSymbol),
    Here(FeatureSymbol),
}

#[derive(Clone, Debug)]
pub struct ComplexReferencePath {
    steps: Vec<ComplexReference>,
}

impl ComplexReferencePath {
    pub fn normalize(&self) -> NormalReferencePath {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NormalReference {
    Child(PartSymbol),
    /// Behaves as a mirror of `Reference::Substructure`
    Substructure(FeatureSymbol),
    Here(FeatureSymbol),
}

#[derive(Clone, Debug)]
pub struct NormalReferencePath {
    steps: Vec<NormalReference>,
}

pub trait Reference: PartialEq + Eq + Copy + Clone {
    fn new_here(symbol: FeatureSymbol) -> Self;
    fn get_here(&self) -> Option<FeatureSymbol>;
    fn is_here(&self) -> bool;
}

impl Reference for ComplexReference {
    fn new_here(symbol: FeatureSymbol) -> Self {
        ComplexReference::Here(symbol)
    }

    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            ComplexReference::Parent
            | ComplexReference::Sibling(_)
            | ComplexReference::Child(_)
            | ComplexReference::Substructure(_) => None,
            ComplexReference::Here(feature_symbol) => Some(*feature_symbol),
        }
    }

    fn is_here(&self) -> bool {
        match self {
            ComplexReference::Parent
            | ComplexReference::Sibling(_)
            | ComplexReference::Child(_)
            | ComplexReference::Substructure(_) => false,
            ComplexReference::Here(_) => true,
        }
    }
}

impl Reference for NormalReference {
    fn new_here(symbol: FeatureSymbol) -> Self {
        NormalReference::Here(symbol)
    }

    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            NormalReference::Child(_) | NormalReference::Substructure(_) => None,
            NormalReference::Here(feature_symbol) => Some(*feature_symbol),
        }
    }

    fn is_here(&self) -> bool {
        match self {
            NormalReference::Child(_) | NormalReference::Substructure(_) => false,
            NormalReference::Here(_) => true,
        }
    }
}

pub trait ReferencePath: Sized {
    type Reference: Reference;

    fn new(steps: impl Into<Vec<Self::Reference>>) -> Self;
    fn steps(&self) -> &Vec<Self::Reference>;
    fn steps_mut(&mut self) -> &mut Vec<Self::Reference>;

    /// This function relies on `Self::push()` and therefore will also
    /// return an `Err(())` if one attempts append while the last value is already
    /// `Here(Symbol)`.
    fn append(&mut self, mut path: Self) -> Result<(), ()> {
        if self.is_final() {
            self.steps_mut().append(path.steps_mut());
            Ok(())
        } else {
            Err(())
        }
    }

    fn is_final(&self) -> bool {
        match self.steps().last() {
            None => false,
            Some(reference) => reference.is_here(),
        }
    }

    /// This function will return an `Err(())` if one attempts to push another
    /// path step when the last value is already `Here(Symbol)`.
    fn push(&mut self, reference: Self::Reference) -> Result<(), ()> {
        if self.is_final() {
            self.steps_mut().push(reference);
            Ok(())
        } else {
            Err(())
        }
    }

    /// This function relies on `Self::push()` and `Self::append()` and
    /// therefore will also return an `Err(())` if one attempts to map
    /// a value after mapping the previous value to `Here(Symbol)`.
    fn map_and_branch<F>(&self, mut f: F) -> Result<Self, ()>
    where
        F: FnMut(&Self::Reference) -> StopOrContinue<Self::Reference, Self>,
    {
        let mut output = Self::new([]);
        for step in self.steps() {
            match f(step) {
                StopOrContinue::Stop(reference) => output.push(reference)?,
                StopOrContinue::Continue(branch_path) => output.append(branch_path)?,
            }
        }

        Ok(output)
    }

    fn pop(&mut self) -> Option<Self::Reference> {
        self.steps_mut().pop()
    }

    fn pop_final(&mut self) -> Option<FeatureSymbol> {
        if self.is_final() {
            match self.pop() {
                Some(reference) => match reference.get_here() {
                    Some(symbol) => Some(symbol),
                    None => panic!("you should never see this error because it is impossible"),
                },
                None => None,
            }
        } else {
            None
        }
    }

    fn split_first(&self) -> Option<(Self::Reference, Self)> {
        match self.steps().split_first() {
            Some((first, rest)) => Some((*first, Self::new(rest))),
            None => None,
        }
    }

    fn into_iter(self) -> ReferencePathIterator<Self> {
        ReferencePathIterator { remaining: self }
    }
}

impl ReferencePath for ComplexReferencePath {
    type Reference = ComplexReference;

    fn new(steps: impl Into<Vec<ComplexReference>>) -> Self {
        Self {
            steps: steps.into(),
        }
    }

    fn steps_mut(&mut self) -> &mut Vec<ComplexReference> {
        &mut self.steps
    }

    fn steps(&self) -> &Vec<ComplexReference> {
        &self.steps
    }
}

impl ReferencePath for NormalReferencePath {
    type Reference = NormalReference;

    fn new(steps: impl Into<Vec<NormalReference>>) -> Self {
        Self {
            steps: steps.into(),
        }
    }

    fn steps_mut(&mut self) -> &mut Vec<NormalReference> {
        &mut self.steps
    }

    fn steps(&self) -> &Vec<NormalReference> {
        &self.steps
    }
}

pub struct ReferencePathIterator<Path: ReferencePath> {
    remaining: Path,
}

impl<Path: ReferencePath> Iterator for ReferencePathIterator<Path> {
    type Item = (Path::Reference, Option<Path::Reference>);

    fn next(&mut self) -> Option<Self::Item> {
        let (first, rest) = self.remaining.split_first()?;
        match rest.split_first() {
            Some((second, _)) => {
                self.remaining = rest;
                Some((first, Some(second)))
            }
            None => {
                self.remaining = Path::new([]);
                Some((first, None))
            }
        }
    }
}

/*
 * ==========
 * RESOLUTION
 * ==========
 */

pub struct Sourced<'a, Root: DataSource> {
    pub root: &'a Root,
    pub path: ComplexReferencePath,
}

pub enum StopOrContinue<A, B> {
    Stop(A),
    Continue(B),
}

impl<A, B> StopOrContinue<A, B> {
    pub const fn stop_value(&self) -> Option<&A> {
        match self {
            StopOrContinue::Stop(v) => Some(v),
            _ => None,
        }
    }

    pub const fn continue_value(&self) -> Option<&B> {
        match self {
            StopOrContinue::Continue(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a, Root: DataSource> Sourced<'a, Root> {}

pub struct Defined<Source: DataSource, Object> {
    pub data_source: Source,
    pub instance: Object,
}

pub struct Referenced<Origin: ReferenceOrigin, Symbol: Clone + Copy> {
    pub reference_point: Origin,
    pub instance: Symbol,
}

pub trait ReferenceOrigin {
    type Output;

    fn resolve_absolute_reference(
        &self,
        reference_path: ComplexReferencePath,
    ) -> Result<Self::Output, ResolutionError>;
}

impl<Source: DataSource> ReferenceOrigin for Defined<Source, StructureSchema> {
    type Output = FeatureSchema;

    fn resolve_absolute_reference(
        &self,
        reference_path: ComplexReferencePath,
    ) -> Result<structure::FeatureSchema, ResolutionError> {
        // for (this_reference, next_reference) in reference_path {
        //     match first {
        //         Reference::Parent => Err(ResolutionError::ImpossibleParent),
        //         Reference::Sibling(part_symbol) => Err(ResolutionError::Placeholder("")),
        //         Reference::Child(part_symbol) => Err(ResolutionError::Placeholder("")),
        //         Reference::Substructure(_) => Err(ResolutionError::Placeholder("")),
        //         Reference::Here(_) => Err(ResolutionError::Placeholder("")),
        //     }
        // }

        // if let Some((first, rest)) = reference_path.first_rest() {
        //     match first {
        //         Reference::Parent => Err(ResolutionError::ImpossibleParent),
        //         Reference::Sibling(part_symbol) => Err(ResolutionError::Placeholder("")),
        //         Reference::Child(part_symbol) => Err(ResolutionError::Placeholder("")),
        //         Reference::Substructure(_) => Err(ResolutionError::Placeholder("")),
        //         Reference::Here(_) => Err(ResolutionError::Placeholder("")),
        //     }
        // } else {
        //     Err(ResolutionError::EmptyPath)
        // }
        //

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

impl DataSource for DataTable {
    type Feature = FeatureSchema;
    type Part = PartSchema;
    type Structure = StructureSchema;

    fn get_feature(&self, feature: &FeatureSymbol) -> Option<&Self::Feature> {
        self.get_schema(*feature)
            .map(|schema| match schema {
                Schema::Feature(schema) => Some(schema),
                _ => None,
            })
            .unwrap_or(None)
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

pub struct ValidatedInstance<'a>(&'a StructureInstance);

impl<'a> ValidatedInstance<'a> {
    pub fn structure(&self) -> &'a StructureInstance {
        self.0
    }
}

pub enum ResolutionError {
    EmptyPath,
    ReferenceTooHigh,
    NoTableEntry(Symbol),
    NonPartSubstructureReference,
    Placeholder(&'static str),
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
