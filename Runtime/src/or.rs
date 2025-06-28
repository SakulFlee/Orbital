#[derive(Debug)]
pub enum Or<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left: Clone, Right: Clone> Clone for Or<Left, Right> {
    fn clone(&self) -> Self {
        match self {
            Or::Left(l) => Or::Left(l.clone()),
            Or::Right(r) => Or::Right(r.clone()),
        }
    }
}

impl<Left: PartialEq, Right: PartialEq> PartialEq for Or<Left, Right> {
    fn eq(&self, other: &Or<Left, Right>) -> bool {
        match self {
            Or::Left(l) => match other {
                Or::Left(o) => l == o,
                Or::Right(_) => false,
            },
            Or::Right(r) => match other {
                Or::Left(_) => false,
                Or::Right(o) => r == o,
            },
        }
    }
}
