use std::collections::HashSet;
use std::{any::TypeId, collections::HashMap, rc::Rc};

mod structure;
pub use crate::structure::*;
mod resolution;
pub use crate::resolution::*;

#[allow(unused)]
fn main() {
    let mut table = DataTable::new();

    let number = table.define(FeatureSchema {
        value_type: FeatureValueSchema::Type(TypeId::of::<usize>()),
    });

    let noun_phrase = table.define(StructureSchema {
        features: HashMap::from([(number.try_into().unwrap(), Relation::MustSelfDefine)]),
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
            Relation::MustInherit(ComplexReferencePath::new([
                ComplexReference::Sibling(sentence_subject.try_into().unwrap()),
                ComplexReference::Here(number.try_into().unwrap()),
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
