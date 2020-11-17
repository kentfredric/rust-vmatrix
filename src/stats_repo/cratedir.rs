use std::{
  ffi::OsString,
  fs, io,
  path::{Path, PathBuf},
};

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

pub(crate) fn sections_in<P, C>(root: P, prefix: C) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
  C: AsRef<str>,
{
  let mut sections = Vec::new();
  for entry_result in std::fs::read_dir(root.as_ref())? {
    let entry_name = entry_result?.file_name();
    let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;
    if let Some(c) = entry_str.strip_prefix(prefix.as_ref()) {
      if c.len() == 1 {
        sections.push(c.to_owned());
      } else {
        return Err(Error::BadSection(entry_name, prefix.as_ref().into(), root.as_ref().into()));
      }
    }
  }
  Ok(sections)
}

pub(crate) fn subsections_in<C, P>(root: P, prefix: C) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
  C: AsRef<str>,
{
  let mut subsections = Vec::new();
  for entry_result in std::fs::read_dir(root.as_ref())? {
    let entry_name = entry_result?.file_name();
    let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;

    if 2 >= entry_str.len() && entry_str.starts_with(prefix.as_ref()) {
      subsections.push(entry_str.to_owned())
    } else {
      return Err(Error::BadSubSection(entry_name, prefix.as_ref().into(), root.as_ref().into()));
    }
  }
  Ok(subsections)
}

pub(crate) fn crates_in<C, P>(root: P, prefix: C) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
  C: AsRef<str>,
{
  let mut crates = Vec::new();
  for entry_result in std::fs::read_dir(root.as_ref())? {
    let entry_name = entry_result?.file_name();
    let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;

    if entry_str.starts_with(prefix.as_ref()) {
      crates.push(entry_str.to_owned())
    } else {
      return Err(Error::BadCrate(entry_name, prefix.as_ref().into(), root.as_ref().into()));
    }
  }
  Ok(crates)
}

#[derive(Debug)]
pub struct InBandDirIterator {
  root:  PathBuf,
  inner: Option<Result<fs::ReadDir, ()>>,
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

#[derive(Debug)]
pub struct SectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
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

#[derive(Debug)]
pub struct SubSectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
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

#[derive(Debug)]
pub struct CrateIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
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

#[derive(Debug)]
pub enum Error {
  NotUnicode(OsString),
  IoError(std::io::Error),
  BadSection(OsString, String, PathBuf),
  NonSection(OsString, String, PathBuf),
  BadSubSection(OsString, String, PathBuf),
  BadCrate(OsString, String, PathBuf),
  BadCrateName(String),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

impl std::fmt::Display for Error {
  fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      | Self::NotUnicode(s) => {
        write!(fmter, "Could not decode {:?} as Unicode", s)
      },
      | Self::IoError(e) => {
        write!(fmter, "IO Error in stats repo directory: {}", e)
      },
      | Self::BadSection(sname, prefix, root) => {
        write!(
          fmter,
          "Directory Section {:?} in {:?} does not satisfy the layout scheme (should be one character after prefix {})",
          sname, root, prefix
        )
      },
      | Self::NonSection(sname, prefix, root) => {
        write!(
          fmter,
          "Directory Section {:?} in {:?} does not satisfy the layout scheme (doesn't start with prefix {})",
          sname, root, prefix
        )
      },

      | Self::BadSubSection(sname, prefix, root) => {
        write!(
          fmter,
          "Subsection {:?} in {:?} does not satisfy the layout scheme (should be 1-or-2 characters, and start with {} \
           )",
          sname, root, prefix
        )
      },
      | Self::BadCrate(sname, prefix, root) => {
        write!(
          fmter,
          "Crate {:?} in {:?} does not satisfy the layout scheme (should start with {})",
          sname, root, prefix
        )
      },
      | Self::BadCrateName(s) => {
        write!(fmter, "Crate name {:?} is illegal, must have at least one character", s)
      },
    }
  }
}
impl std::error::Error for Error {}
