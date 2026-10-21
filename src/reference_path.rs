use crate::{Complex, Descent, Reference};

#[derive(Clone, Debug)]
pub struct ComplexPath {
    path_steps: Vec<Complex>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PathError {
    IllegalRootParent,
    IllegalRootSibling,
    ContinuesAfterHere,
}

impl ComplexPath {
    pub fn normalize(&self) -> Result<ComplexPath, PathError> {
        let mut stack = Vec::new();

        for &path_step in &self.path_steps {
            stack.push(path_step);

            while stack.len() >= 2 {
                let (a, b) = (stack[stack.len() - 2], stack[stack.len() - 1]);

                let replacement = match (a, b) {
                    (Complex::Sibling(_), Complex::Sibling(s)) => Some(Complex::Sibling(s)),
                    (Complex::Child(_), Complex::Sibling(s)) => Some(Complex::Child(s)),
                    (Complex::Child(_), Complex::Parent) => None,
                    (Complex::Parent, Complex::Child(s)) => Some(Complex::Sibling(s)),
                    (Complex::Sibling(_), Complex::Parent) => Some(Complex::Parent),
                    (Complex::Here(_), _) | (Complex::HereInSubstructure(_), _) => {
                        return Err(PathError::ContinuesAfterHere);
                    }
                    _ => break,
                };

                stack.pop();
                stack.pop();

                match replacement {
                    Some(r) => stack.push(r),
                    None => (),
                };
            }
        }

        Ok(ComplexPath { path_steps: stack })
    }

    pub fn is_descending_option(&self) -> Option<PathError> {
        for step in &self.path_steps {
            match step {
                Complex::Parent => return Some(PathError::IllegalRootParent),
                Complex::Sibling(_) => return Some(PathError::IllegalRootParent),
                _ => (),
            }
        }

        None
    }

    pub fn is_descending_bool(&self) -> bool {
        for step in &self.path_steps {
            match step {
                Complex::Parent | Complex::Sibling(_) => return false,
                _ => (),
            }
        }

        true
    }

    pub fn into_descent(&self) -> Result<DescentPath, PathError> {
        let normalized = self.normalize()?;

        match normalized.is_descending_option() {
            Some(err) => Err(err),
            None => Ok(DescentPath {
                path_steps: normalized
                    .path_steps
                    .iter()
                    .map(|&complex| match complex {
                        Complex::Child(s) => Descent::Child(s),
                        Complex::HereInSubstructure(s) => Descent::HereInSubstructure(s),
                        Complex::Here(s) => Descent::Here(s),
                        _ => panic!("this should be impossible @ ComplexPath::into_descent()"),
                    })
                    .collect(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DescentPath {
    path_steps: Vec<Descent>,
}

pub trait ReferencePath: Sized + Clone {
    type Reference: Reference;

    fn new(path_steps: impl Into<Vec<Self::Reference>>) -> Self;
    fn path_steps(&self) -> &Vec<Self::Reference>;
    fn path_steps_mut(&mut self) -> &mut Vec<Self::Reference>;

    /// This function relies on `Self::push()` and therefore will also
    /// return an `Err(())` if one attempts append while the last value is already
    /// `Here(Symbol)`.
    fn append(&mut self, mut path: Self) -> Result<(), ()> {
        if self.terminates_on_here() {
            Err(())
        } else {
            self.path_steps_mut().append(path.path_steps_mut());
            Ok(())
        }
    }

    fn terminates_on_here(&self) -> bool {
        match self.path_steps().last() {
            None => false,
            Some(reference) => reference.is_here(),
        }
    }

    /// This function will return an `Err(())` if one attempts to push another
    /// path step when the last value is already a "Here" value.
    fn push(&mut self, reference: Self::Reference) -> Result<(), ()> {
        if self.terminates_on_here() {
            Err(())
        } else {
            self.path_steps_mut().push(reference);
            Ok(())
        }
    }

    /// This function relies on `Self::push()` and `Self::append()` and
    /// therefore will also return `None` if one attempts to map
    /// a value after mapping the previous value to a "Here" value.
    fn map_and_branch<F>(&self, mut f: F) -> Option<Self>
    where
        F: FnMut(&Self::Reference) -> StopOrContinue<Self::Reference, Self>,
    {
        let mut output = Self::new([]);
        for step in self.path_steps() {
            match f(step) {
                StopOrContinue::Stop(reference) => output.push(reference).ok()?,
                StopOrContinue::Continue(branch_path) => output.append(branch_path).ok()?,
            }
        }

        Some(output)
    }

    fn pop(&mut self) -> Option<Self::Reference> {
        self.path_steps_mut().pop()
    }

    fn pop_final(&mut self) -> Option<Self::Reference> {
        if self.terminates_on_here() {
            match self.pop() {
                Some(reference) => match reference.get_here() {
                    Some(_) => Some(reference),
                    None => panic!("you should never see this error because it is impossible"),
                },
                None => None,
            }
        } else {
            None
        }
    }

    fn first_rest(&self) -> Option<(Self::Reference, Self)> {
        match self.path_steps().split_first() {
            Some((first, rest)) => Some((*first, Self::new(rest))),
            None => None,
        }
    }

    fn into_iter(&self) -> ReferencePathIterator<Self> {
        ReferencePathIterator {
            remaining: self.clone(),
        }
    }

    fn replace_here(&self, path: Self) -> Option<Self> {
        let mut new = self.clone();
        match new.pop_final() {
            Some(_) => {
                new.append(path).ok()?;
                Some(new)
            }
            None => None,
        }
    }

    fn insert_before_here(&self, path: Self) -> Option<Self> {
        if !path.terminates_on_here() {
            let mut new = self.clone();
            match new.pop_final() {
                Some(here) => {
                    new.append(path).ok()?;

                    new.push(here).ok()?;
                    Some(new)
                }
                None => None,
            }
        } else {
            None
        }
    }
}

impl ReferencePath for ComplexPath {
    type Reference = Complex;

    fn new(path_steps: impl Into<Vec<Complex>>) -> Self {
        Self {
            path_steps: path_steps.into(),
        }
    }

    fn path_steps_mut(&mut self) -> &mut Vec<Complex> {
        &mut self.path_steps
    }

    fn path_steps(&self) -> &Vec<Complex> {
        &self.path_steps
    }
}

impl ReferencePath for DescentPath {
    type Reference = Descent;

    fn new(path_steps: impl Into<Vec<Descent>>) -> Self {
        Self {
            path_steps: path_steps.into(),
        }
    }

    fn path_steps_mut(&mut self) -> &mut Vec<Descent> {
        &mut self.path_steps
    }

    fn path_steps(&self) -> &Vec<Descent> {
        &self.path_steps
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
