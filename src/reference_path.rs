use crate::{ComplexReference, FeatureSymbol, NormalReference, Reference};

#[derive(Clone, Debug)]
pub struct ComplexReferencePath {
    steps: Vec<ComplexReference>,
}

impl ComplexReferencePath {
    pub fn normalize(&self) -> NormalReferencePath {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct NormalReferencePath {
    steps: Vec<NormalReference>,
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
    /// path step when the last value is already a "Here" value.
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
    /// a value after mapping the previous value to a "Here" value.
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

    fn first_rest(&self) -> Option<(Self::Reference, Self)> {
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
        let (first, rest) = self.remaining.first_rest()?;
        match rest.first_rest() {
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
