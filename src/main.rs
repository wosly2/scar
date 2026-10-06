use std::{any::TypeId, collections::HashMap, rc::Rc};

mod structure;
pub use crate::structure::*;
mod resolution;
pub use crate::resolution::*;

#[allow(unused)]
fn main() {
    let mut table = DataTable::new();

    let number = table.define_feature(FeatureSchema {
        value_type: FeatureValueSchema::Type(TypeId::of::<usize>()),
    });

    let noun_phrase = table.define_structure(StructureSchema {
        features: HashMap::from([(number, Relation::MustSelfDefine)]),
        parts: HashMap::new(),
    });

    let verb_phrase = table.define_structure(StructureSchema {
        features: HashMap::from([(
            number,
            Relation::MustInherit(Reference::Sibling(
                PartSymbol::new("subject"),
                Some(Rc::new(Reference::Finally(number))),
            )),
        )]),
        parts: HashMap::new(),
    });

    let sentence = table.define_structure(StructureSchema {
        features: HashMap::new(),
        parts: HashMap::from([
            (
                PartSymbol::new("subject"),
                PartSchema {
                    features: HashMap::new(),
                    structure: noun_phrase,
                    presence: Presence::Required,
                },
            ),
            (
                PartSymbol::new("verb"),
                PartSchema {
                    features: HashMap::new(),
                    structure: verb_phrase,
                    presence: Presence::Required,
                },
            ),
            (
                PartSymbol::new("object"),
                PartSchema {
                    features: HashMap::new(),
                    structure: noun_phrase,
                    presence: Presence::Optional,
                },
            ),
        ]),
    });

    let mut example_sentence = StructureInstance {
        schema: sentence,
        features: HashMap::new(),
        parts: HashMap::from([
            (
                PartSymbol::new("subject"),
                PartInstance {
                    features: HashMap::new(),
                    structure: StructureInstance {
                        schema: noun_phrase,
                        features: HashMap::from([FeatureInstance {
                            schema: number,
                            value: FeatureValueInstance::Type(Rc::new(0 as usize)),
                        }
                        .key_pair()]),
                        parts: HashMap::new(),
                    },
                },
            ),
            // missing other parts here, imagine they exist
        ]),
    };

    println!("Hello, world!");
}
