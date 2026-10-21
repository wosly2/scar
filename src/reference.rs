use crate::{ComplexPath, FeatureSymbol, PartSymbol};

#[derive(Clone, Copy, Debug)]
pub enum Presence {
    Required,
    Optional,
}

#[derive(Clone, Debug)]
pub enum Relation {
    MustSelfDefine,
    PrefersSelfDefined(ComplexPath),
    MustInherit(ComplexPath),
    PrefersInherited(ComplexPath),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Complex {
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
pub enum Descent {
    Child(PartSymbol),
    /// Behaves as a mirror of `Reference::HereInSubstructure`
    HereInSubstructure(FeatureSymbol),
    Here(FeatureSymbol),
}
pub trait Reference: PartialEq + Eq + Copy + Clone {
    fn get_here(&self) -> Option<FeatureSymbol>;
    fn is_here(&self) -> bool;
}

impl Reference for Complex {
    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            Complex::Parent | Complex::Sibling(_) | Complex::Child(_) => None,
            Complex::Here(symbol) | Complex::HereInSubstructure(symbol) => Some(*symbol),
        }
    }

    fn is_here(&self) -> bool {
        match self {
            Complex::Parent | Complex::Sibling(_) | Complex::Child(_) => false,
            Complex::Here(_) | Complex::HereInSubstructure(_) => true,
        }
    }
}
impl Reference for Descent {
    fn get_here(&self) -> Option<FeatureSymbol> {
        match self {
            Descent::Child(_) => None,
            Descent::Here(symbol) | Descent::HereInSubstructure(symbol) => Some(*symbol),
        }
    }

    fn is_here(&self) -> bool {
        match self {
            Descent::Child(_) => false,
            Descent::Here(_) | Descent::HereInSubstructure(_) => true,
        }
    }
}
