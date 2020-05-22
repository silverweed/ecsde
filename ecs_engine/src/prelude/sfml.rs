#![cfg(feature = "use-sfml")]

// Used to wrap an SfBox inside a struct with a lifetime.
// This is done to allow the external API to be as general as possible
// (different backends may give an explicit lifetime to resources)
#[macro_export]
macro_rules! sf_wrap {
    ($typename: ident, $sftypename: ty) => {
        pub struct $typename<'a> {
            pub wrapped: ::sfml::system::SfBox<$sftypename>,
            _marker: &'a std::marker::PhantomData<()>,
        }

        impl $typename<'_> {
            pub fn from_file(fname: &str) -> Option<Self> {
                Some($typename {
                    wrapped: <$sftypename>::from_file(fname)?,
                    _marker: &std::marker::PhantomData,
                })
            }
        }

        impl std::ops::Deref for $typename<'_> {
            type Target = $sftypename;

            fn deref(&self) -> &Self::Target {
                &self.wrapped
            }
        }

        impl std::ops::DerefMut for $typename<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.wrapped
            }
        }
    };
}
