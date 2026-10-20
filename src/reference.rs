use crate::{ComplexReferencePath, FeatureSymbol, PartSymbol};

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
    HereInSubstructure(FeatureSymbol),
    Here(FeatureSymbol),
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NormalReference {
    Child(PartSymbol),
    /// Behaves as a mirror of `Reference::HereInSubstructure`
    HereInSubstructure(FeatureSymbol),
    Here(FeatureSymbol),
}
pub trait Reference: PartialEq + Eq + Copy + Clone {
    fn get_here(&self) -> Option<FeatureSymbol>;
    fn is_here(&self) -> bool;
}

impl Reference for ComplexReference {
    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            ComplexReference::Parent
            | ComplexReference::Sibling(_)
            | ComplexReference::Child(_) => None,
            ComplexReference::Here(symbol) | ComplexReference::HereInSubstructure(symbol) => {
                Some(*symbol)
            }
        }
    }

    fn is_here(&self) -> bool {
        match self {
            ComplexReference::Parent
            | ComplexReference::Sibling(_)
            | ComplexReference::Child(_) => false,
            ComplexReference::Here(_) | ComplexReference::HereInSubstructure(_) => true,
        }
    }
}
impl Reference for NormalReference {
    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            NormalReference::Child(_) => None,
            NormalReference::Here(symbol) | NormalReference::HereInSubstructure(symbol) => {
                Some(*symbol)
            }
        }
    }

    fn is_here(&self) -> bool {
        match self {
            NormalReference::Child(_) => false,
            NormalReference::Here(_) | NormalReference::HereInSubstructure(_) => true,
        }
    }
}
