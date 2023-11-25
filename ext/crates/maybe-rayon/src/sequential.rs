pub mod prelude {
    pub trait MaybeParallelIterator: Iterator {}

    pub trait MaybeIndexedParallelIterator: Iterator {}

    pub trait MaybeIntoParallelIterator: IntoIterator {
        type Iter;

        fn maybe_into_par_iter(self) -> Self::Iter;
    }

    pub trait MaybeIntoParallelRefMutIterator<'data> {
        type Iter;

        fn maybe_par_iter_mut(&'data mut self) -> Self::Iter;
    }

    // Implementations

    impl<I: Iterator> MaybeParallelIterator for I {}

    impl<I: Iterator> MaybeIndexedParallelIterator for I {}

    impl<I: IntoIterator> MaybeIntoParallelIterator for I {
        type Iter = Self::IntoIter;

        fn maybe_into_par_iter(self) -> Self::Iter {
            self.into_iter()
        }
    }

    impl<'data, I: 'data + ?Sized> MaybeIntoParallelRefMutIterator<'data> for I
    where
        &'data mut I: IntoIterator,
    {
        type Iter = <&'data mut I as IntoIterator>::IntoIter;

        fn maybe_par_iter_mut(&'data mut self) -> Self::Iter {
            self.into_iter()
        }
    }
}

pub struct Scope<'scope>(&'scope ());

impl<'scope> Scope<'scope> {
    pub fn spawn<BODY>(&self, body: BODY)
    where
        BODY: FnOnce(&Scope<'scope>) + Send + 'scope,
    {
        body(self)
    }
}

pub fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    RA: Send,
    RB: Send,
{
    (oper_a(), oper_b())
}

pub fn scope<'scope, OP, R>(op: OP) -> R
where
    OP: FnOnce(&Scope<'scope>) -> R + Send,
    R: Send,
{
    op(&Scope(&()))
}

pub fn in_place_scope<'scope, OP, R>(op: OP) -> R
where
    OP: FnOnce(&Scope<'scope>) -> R,
{
    op(&Scope(&()))
}
