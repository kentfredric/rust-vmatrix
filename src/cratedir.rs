use std::{
  ffi::OsString,
  fs, io,
  path::{Path, PathBuf},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Could not decode {0:?} as Unicode")]
  NotUnicode(OsString),
  #[error("IO Error in stats repo directory: {0}")]
  IoError(#[from] std::io::Error),
  #[error(
    "Directory Section {0:?} in {2:?} does not satisfy the layout scheme (should be one character after prefix {1})"
  )]
  BadSection(OsString, String, PathBuf),
  #[error("Directory Section {0:?} in {2:?} does not satisfy the layout scheme (doesn't start with prefix {1})")]
  NonSection(OsString, String, PathBuf),
  #[error(
    "Subsection {0:?} in {2:?} does not satisfy the layout scheme (should be 1-or-2 characters and start with {1})"
  )]
  BadSubSection(OsString, String, PathBuf),
  #[error("Crate {0:?} in {2:?} does not satisfy the layout scheme (should start with {1})")]
  BadCrate(OsString, String, PathBuf),
  #[error("Crate name {0:?} is illegal, must have at least one character")]
  BadCrateName(String),
}

#[derive(Debug)]
pub struct InBandDirIterator {
  root:  PathBuf,
  inner: Option<Result<fs::ReadDir, ()>>,
}

#[derive(Debug)]
pub struct SectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
pub struct SubSectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
pub struct CrateIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

pub(crate) fn crate_rpath<P, C>(prefix: P, crate_name: C) -> Result<PathBuf, Error>
where
  P: AsRef<str>,
  C: AsRef<str>,
{
  let mut crate_chars = crate_name.as_ref().chars();
  let first = crate_chars.next().ok_or_else(|| Error::BadCrateName(crate_name.as_ref().to_owned()))?;
  let nibble = match crate_chars.next() {
    | Some(c) => format!("{}{}", first, c),
    | None => first.to_string(),
  };
  Ok(PathBuf::from(format!("{}{}", prefix.as_ref(), first)).join(nibble).join(crate_name.as_ref()))
}

pub(crate) fn crate_path<R, P, C>(root: R, prefix: P, crate_name: C) -> Result<PathBuf, Error>
where
  R: AsRef<Path>,
  P: AsRef<str>,
  C: AsRef<str>,
{
  Ok(root.as_ref().join(crate_rpath(prefix, crate_name)?))
}

impl InBandDirIterator {
  pub fn new<R>(root: R) -> InBandDirIterator
  where
    R: AsRef<Path>,
  {
    InBandDirIterator { root: root.as_ref().to_path_buf(), inner: Option::None }
  }
}

impl Iterator for InBandDirIterator {
  type Item = Result<fs::DirEntry, io::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    // This will only ever not be a None
    // when initializing ReadDir and read_dir() fails
    let mut error = None::<Self::Item>;
    let root = &self.root;
    match self.inner.get_or_insert_with(|| {
      match fs::read_dir(&root) {
        | Ok(inner) => Ok(inner),
        | Err(err) => {
          // Stash the failure
          error = Some(Err(err));
          Err(())
        },
      }
    }) {
      | Ok(inner) => inner,
      // Returns None on non-first calls if read_dir failed
      | Err(()) => return error,
    }
    .next()
  }
}

impl SectionIterator {
  pub fn new<R, P>(root: R, prefix: P) -> SectionIterator
  where
    R: AsRef<Path>,
    P: AsRef<str>,
  {
    SectionIterator {
      root:   root.as_ref().to_path_buf(),
      prefix: prefix.as_ref().to_string(),
      inner:  InBandDirIterator::new(root),
    }
  }
}

impl Iterator for SectionIterator {
  type Item = Result<String, self::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;

      if let Some(c) = entry_str.strip_prefix(&self.prefix) {
        if c.len() == 1 {
          Ok(c.to_owned())
        } else {
          Err(Error::BadSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
        }
      } else {
        Err(Error::NonSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}

impl SubSectionIterator {
  pub fn new<R, P>(root: R, prefix: P) -> SubSectionIterator
  where
    R: AsRef<Path>,
    P: AsRef<str>,
  {
    SubSectionIterator {
      root:   root.as_ref().to_path_buf(),
      prefix: prefix.as_ref().to_string(),
      inner:  InBandDirIterator::new(root),
    }
  }
}

impl Iterator for SubSectionIterator {
  type Item = Result<String, self::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;

      if 2 >= entry_str.len() && entry_str.starts_with(&self.prefix) {
        Ok(entry_str.to_owned())
      } else {
        Err(Error::BadSubSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}

impl CrateIterator {
  pub fn new<R, P>(root: R, prefix: P) -> CrateIterator
  where
    R: AsRef<Path>,
    P: AsRef<str>,
  {
    CrateIterator {
      root:   root.as_ref().to_path_buf(),
      prefix: prefix.as_ref().to_string(),
      inner:  InBandDirIterator::new(root),
    }
  }
}

impl Iterator for CrateIterator {
  type Item = Result<String, self::Error>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;

      if entry_str.starts_with(&self.prefix) {
        Ok(entry_str.to_owned())
      } else {
        Err(Error::BadCrate(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}
