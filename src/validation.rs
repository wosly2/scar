use crate::{ResolutionError, StructureInstance};

pub struct ValidatedInstance<'a>(&'a StructureInstance);

impl<'a> ValidatedInstance<'a> {
    pub fn structure(&self) -> &'a StructureInstance {
        self.0
    }
}

pub enum ValiditationError {
    Resolution(ResolutionError),
    Placeholder(&'static str),
}

impl StructureInstance {
    pub fn validate<'a>(
        structure: &'a StructureInstance,
    ) -> Result<ValidatedInstance<'a>, ValiditationError> {
        Ok(ValidatedInstance(structure))
    }
}
