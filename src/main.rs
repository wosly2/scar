use std::collections::HashSet;
use std::{any::TypeId, collections::HashMap, rc::Rc};

mod data_source;
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

macro_rules! t {
    [$a:expr] => {
        $a.try_into().unwrap()
    };
}

#[allow(unused)]
fn main() {
    let mut table = DataTable::new();

    let number = table.define(FeatureSchema {
        value_type: FeatureValueSchema::Type(TypeId::of::<usize>()),
    });

    let noun_phrase = table.define(StructureSchema {
        features: HashMap::from([(t![number], Relation::MustSelfDefine)]),
        parts: HashSet::new(),
    });

    let sentence_subject = table.define(PartSchema {
        features: HashMap::new(),
        structure: noun_phrase.try_into().unwrap(),
        presence: Presence::Required,
    });

    let verb_phrase = table.define(StructureSchema {
        features: HashMap::from([(
            number.try_into().unwrap(),
            Relation::MustInherit(ComplexPath::new([
                Complex::Sibling(sentence_subject.try_into().unwrap()),
                Complex::Here(number.try_into().unwrap()),
            ])),
        )]),
        parts: HashSet::new(),
    });

    let sentence_verb = table.define(PartSchema {
        features: HashMap::new(),
        structure: verb_phrase.try_into().unwrap(),
        presence: Presence::Required,
    });

    let sentence_object = table.define(PartSchema {
        features: HashMap::new(),
        structure: noun_phrase.try_into().unwrap(),
        presence: Presence::Optional,
    });

    let sentence = table.define(StructureSchema {
        features: HashMap::new(),
        parts: HashSet::from([
            sentence_subject.try_into().unwrap(),
            sentence_verb.try_into().unwrap(),
            sentence_object.try_into().unwrap(),
        ]),
    });

    let mut example_sentence = StructureInstance {
        schema: sentence.try_into().unwrap(),
        features: HashMap::new(),
        parts: HashMap::from([
            (
                sentence_subject.try_into().unwrap(),
                PartInstance {
                    features: HashMap::new(),
                    structure: StructureInstance {
                        schema: noun_phrase.try_into().unwrap(),
                        features: HashMap::from([FeatureInstance {
                            schema: number.try_into().unwrap(),
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
