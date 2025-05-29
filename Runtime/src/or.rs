#[derive(Debug)]
pub enum Or<Left, Right> {
    Left(Left),
    Right(Right),
}
