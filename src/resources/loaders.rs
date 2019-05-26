pub trait Resource_Loader<'l, R> {
    type Args: ?Sized;

    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}
