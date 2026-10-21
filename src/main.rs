use std::collections::HashSet;
use std::{any::TypeId, collections::HashMap, rc::Rc};

mod data_source;
mod macros;
mod reference;
mod reference_path;
mod resolution;
mod structure;
mod validation;

pub use data_source::*;
pub use reference::*;
pub use reference_path::*;
pub use resolution::*;
pub use structure::*;
pub use validation::*;

#[allow(unused)]
fn main() {
    let mut table = DataTable::new();

    let number = table.define(FeatureSchema {
        value_type: FeatureValueSchema::Type(TypeId::of::<usize>()),
    });

    let noun_phrase = table.define(StructureSchema {
        features: HashMap::from([(iu![number], Relation::MustSelfDefine)]),
        parts: HashSet::new(),
    });

    let sentence_subject = table.define(PartSchema {
        features: HashMap::new(),
        structure: iu![noun_phrase],
        presence: Presence::Required,
    });

    let verb_phrase = table.define(StructureSchema {
        features: HashMap::from([(
            iu![number],
            Relation::MustInherit(ComplexPath::new([
                Complex::Sibling(iu![sentence_subject]),
                Complex::Here(iu![number]),
            ])),
        )]),
        parts: HashSet::new(),
    });

    let sentence_verb = table.define(PartSchema {
        features: HashMap::new(),
        structure: iu![verb_phrase],
        presence: Presence::Required,
    });

    let sentence_object = table.define(PartSchema {
        features: HashMap::new(),
        structure: iu![noun_phrase],
        presence: Presence::Optional,
    });

    let sentence = table.define(StructureSchema {
        features: HashMap::new(),
        parts: HashSet::from([
            iu![sentence_subject],
            iu![sentence_verb],
            iu![sentence_object],
        ]),
    });

    let mut example_sentence = StructureInstance {
        schema: iu![sentence],
        features: HashMap::new(),
        parts: HashMap::from([
            (
                iu![sentence_subject],
                PartInstance {
                    features: HashMap::new(),
                    structure: StructureInstance {
                        schema: iu![noun_phrase],
                        features: HashMap::from([FeatureInstance {
                            schema: iu![number],
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
