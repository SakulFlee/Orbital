#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Or<Left, Right> {
    Left(Left),
    Right(Right),
}
