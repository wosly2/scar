use crate::{
    DataSource, DataTable, Descent, DescentPath, PartSchema, ReferencePath, Relation,
    StructureSchema, Symbol,
};

pub struct Defined<'a, Source: DataSource, Object> {
    pub data_source: &'a Source,
    pub instance: &'a Object,
}

impl<'a, Source: DataSource, Object> Defined<'a, Source, Object> {
    pub fn define<T>(&self, object: &'a T) -> Defined<'a, Source, T> {
        Defined {
            data_source: self.data_source,
            instance: object,
        }
    }
}

pub struct Referenced<Origin: ReferenceOrigin> {
    pub origin: Origin,
    pub path: DescentPath,
}

impl<Origin: ReferenceOrigin> Referenced<Origin> {
    pub fn resolve(&self) -> Result<Origin::Output, ResolutionError> {
        self.origin.resolve_absolute_reference(&self.path)
    }
}

pub trait ReferenceOrigin {
    type Output;

    fn resolve_absolute_reference(
        &self,
        reference_path: &DescentPath,
    ) -> Result<Self::Output, ResolutionError>;
}

impl<'a> ReferenceOrigin for Defined<'a, DataTable, StructureSchema> {
    type Output = &'a Relation;

    fn resolve_absolute_reference(
        &self,
        reference_path: &DescentPath,
    ) -> Result<&'a Relation, ResolutionError> {
        if let Some((reference, path)) = reference_path.first_rest() {
            match reference {
                Descent::Child(part_symbol) => self
                    .define(
                        self.data_source
                            .get_part(&part_symbol)
                            .ok_or(ResolutionError::NoTableEntry(Symbol::Part(part_symbol)))?,
                    )
                    .resolve_absolute_reference(&path),
                Descent::HereInSubstructure(_) => Err(ResolutionError::NonPartHereInSubstructure),
                Descent::Here(feature_symbol) => self.instance.features.get(&feature_symbol).ok_or(
                    ResolutionError::NoTableEntry(Symbol::Feature(feature_symbol)),
                ),
            }
        } else {
            Err(ResolutionError::EarlyPathTermination)
        }
    }
}

impl<'a> ReferenceOrigin for Defined<'a, DataTable, PartSchema> {
    type Output = &'a Relation;

    fn resolve_absolute_reference(
        &self,
        reference_path: &DescentPath,
    ) -> Result<&'a Relation, ResolutionError> {
        if let Some((reference, path)) = reference_path.first_rest() {
            match reference {
                Descent::Child(part_symbol) => self
                    .define(
                        self.data_source
                            .get_part(&part_symbol)
                            .ok_or(ResolutionError::NoTableEntry(Symbol::Part(part_symbol)))?,
                    )
                    .resolve_absolute_reference(&path),
                Descent::HereInSubstructure(feature_symbol) => {
                    // first get the internal structure schema
                    self.define(
                        self.data_source
                            .get_structure(&self.instance.structure)
                            .ok_or(ResolutionError::NoTableEntry(Symbol::Structure(
                                self.instance.structure,
                            )))?,
                    )
                    // now get its feature
                    .instance
                    .features
                    .get(&feature_symbol)
                    .ok_or(ResolutionError::NoTableEntry(Symbol::Feature(
                        feature_symbol,
                    )))
                }
                Descent::Here(feature_symbol) => {
                    // just grab our own feature
                    self.instance.features.get(&feature_symbol).ok_or(
                        ResolutionError::NoTableEntry(Symbol::Feature(feature_symbol)),
                    )
                }
            }
        } else {
            Err(ResolutionError::EarlyPathTermination)
        }
    }
}

pub enum ResolutionError {
    EarlyPathTermination,
    IllegalAscent,
    NoTableEntry(Symbol),
    NonPartHereInSubstructure,
    Placeholder(&'static str),
}
