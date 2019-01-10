// Copyright (c) 2015 Aaron Power
// Use of this source code is governed by the APACHE2.0/MIT licence that can be
// found in the LICENCE-{APACHE/MIT} file.

use std::collections::{btree_map, BTreeMap};
use std::iter::IntoIterator;
use std::ops::{AddAssign, Deref, DerefMut};

use rayon::prelude::*;

use crate::config::Config;
use super::{Language, LanguageType};
use crate::utils;

/// A newtype representing a list of languages counted in the provided
/// directory.
/// ([_List of
/// Languages_](https://github.com/Aaronepower/tokei#supported-languages))
#[derive(Debug, Default, Serialize)]
pub struct Languages {
    inner: BTreeMap<LanguageType, Language>,
}

impl<'de> serde::Deserialize<'de> for Languages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let map = <_>::deserialize(deserializer)?;

            Ok(Self::from_previous(map))
        }
}

impl Languages {

    fn from_previous(map: BTreeMap<LanguageType, Language>) -> Self {
        use std::collections::btree_map::Entry::*;
        let mut _self = Self::new();

        for (name, input_language) in map {
            match _self.entry(name) {
                Occupied(mut entry) => {
                    *entry.get_mut() += input_language;
                }
                Vacant(entry) => {
                    entry.insert(input_language);
                }
            }
        }
        _self
    }

    /// Populates the `Languages` struct with statistics about languages
    /// provided by [`Language`].
    ///
    /// Takes a `&[&str]` of paths to recursively traverse, paths can be
    /// relative, absolute or glob paths. a second `&[&str]` of paths to ignore,
    /// these strings use the `.gitignore` syntax, such as `target`
    /// or `**/*.bk`.
    ///
    /// ```no_run
    /// use tokei::{Config, Languages};
    ///
    /// let mut languages = Languages::new();
    /// languages.get_statistics(&["."], &[".git", "target"], &Config::default());
    /// ```
    ///
    /// [`Language`]: struct.Language.html
    pub fn get_statistics(&mut self,
                          paths: &[&str],
                          ignored: &[&str],
                          config: &Config)
    {
        utils::fs::get_all_files(paths, ignored, &mut self.inner, config);
        self.inner.par_iter_mut().for_each(|(_, l)| l.total());
    }

    /// Constructs a new, Languages struct. Languages is always empty and does
    /// not allocate.
    ///
    /// ```
    /// # use tokei::*;
    /// let languages = Languages::new();
    /// ```
    pub fn new() -> Self {
        Languages::default()
    }
}

impl IntoIterator for Languages {
    type Item = <BTreeMap<LanguageType, Language> as IntoIterator>::Item;
    type IntoIter =
        <BTreeMap<LanguageType, Language> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Languages {
    type Item = (&'a LanguageType, &'a Language);
    type IntoIter = btree_map::Iter<'a, LanguageType, Language>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Languages {
    type Item = (&'a LanguageType, &'a mut Language);
    type IntoIter = btree_map::IterMut<'a, LanguageType, Language>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl AddAssign<BTreeMap<LanguageType, Language>> for Languages {
    fn add_assign(&mut self, rhs: BTreeMap<LanguageType, Language>) {

        for (name, language) in rhs {

            if let Some(result) = self.inner.get_mut(&name) {
                *result += language;
            }
        }
    }
}

impl Deref for Languages {
    type Target = BTreeMap<LanguageType, Language>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Languages {
    fn deref_mut(&mut self) -> &mut BTreeMap<LanguageType, Language> {
        &mut self.inner
    }
}
