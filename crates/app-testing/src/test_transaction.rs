use {
    futures::executor::block_on,
    sqlx::{Database, Transaction},
    std::ops::{Deref, DerefMut}
};

/// Transaction that executes the rollback
///
/// !!! Important !!!
///
/// *block_on()* is used in Drop trait, which can lead to blocking of the
/// current thread.
pub struct TestTransaction<'a, D: Database>(Option<Transaction<'a, D>>);

impl<'a, D: Database> TestTransaction<'a, D> {
    pub fn new(tx: Transaction<'a, D>) -> Self {
        Self(tx.into())
    }
}

impl<'a, D: Database> Deref for TestTransaction<'a, D> {
    type Target = Transaction<'a, D>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<D: Database> DerefMut for TestTransaction<'_, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<D: Database> Drop for TestTransaction<'_, D> {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            block_on(tx.rollback()).unwrap();
        }
    }
}
