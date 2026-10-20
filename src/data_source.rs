use crate::{
    DataTable, FeatureSchema, FeatureSymbol, PartSchema, PartSymbol, Schema, StructureSchema,
    StructureSymbol,
};

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
