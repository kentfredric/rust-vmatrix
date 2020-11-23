use std::{
  ffi::OsString,
  fs, io,
  path::{Path, PathBuf},
};

#[derive(thiserror::Error, Debug)]
pub enum CrateDirError {
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
struct InBandDirIterator {
  root:  PathBuf,
  inner: Option<Result<fs::ReadDir, ()>>,
}

#[derive(Debug)]
pub(crate) struct SectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
pub(crate) struct SubSectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
pub(crate) struct CrateIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

pub struct CrateDir {
  root:   PathBuf,
  prefix: String,
}

impl CrateDir {
  pub fn new(root: &Path, prefix: &str) -> Self { CrateDir { root: root.to_owned(), prefix: prefix.to_owned() } }

  fn crate_first(&self, crate_name: &str) -> Result<String, CrateDirError> {
    Ok(crate_name.chars().next().ok_or_else(|| CrateDirError::BadCrateName(crate_name.to_owned()))?.to_string())
  }

  pub fn section_name(&self, crate_name: &str) -> Result<String, CrateDirError> {
    Ok([self.prefix.to_string(), self.crate_first(crate_name)?].concat())
  }

  pub fn subsection_name(&self, crate_name: &str) -> Result<String, CrateDirError> {
    let first = self.crate_first(crate_name)?;
    match crate_name.chars().nth(1) {
      | None => Ok(first),
      | Some(c) => Ok([first, c.to_string()].concat()),
    }
  }

  pub fn path_to(&self, crate_name: &str) -> Result<PathBuf, CrateDirError> {
    Ok(PathBuf::from(self.section_name(crate_name)?).join(self.subsection_name(crate_name)?).join(crate_name))
  }

  pub fn path_to_file(&self, crate_name: &str, file_name: &str) -> Result<PathBuf, CrateDirError> {
    self.path_to(crate_name).map(|x| x.join(file_name))
  }

  pub fn abs_path_to(&self, crate_name: &str) -> Result<PathBuf, CrateDirError> {
    self.path_to(crate_name).map(|x| self.root.join(x))
  }

  pub fn abs_path_to_file(&self, crate_name: &str, file_name: &str) -> Result<PathBuf, CrateDirError> {
    self.abs_path_to(crate_name).map(|x| x.join(file_name))
  }

  pub fn section_ids(&self) -> Box<dyn Iterator<Item = Result<String, CrateDirError>>> {
    Box::new(SectionIterator::new(self.root.to_owned(), self.prefix.to_owned()))
  }

  pub fn section_names(&self) -> Box<dyn Iterator<Item = Result<String, CrateDirError>>> {
    let prefix = self.prefix.to_owned();
    Box::new(self.section_ids().map(move |r| r.map(|s| [prefix.to_string(), s].concat())))
  }

  pub fn subsections_in(&self, section_id: &str) -> Box<dyn Iterator<Item = Result<String, CrateDirError>>> {
    let section_name = [self.prefix.to_string(), section_id.to_string()].concat();
    Box::new(SubSectionIterator::new(self.root.join(section_name), section_id))
  }

  pub fn subsection_ids(&self) -> Box<dyn Iterator<Item = Result<String, CrateDirError>> + '_> {
    use either::Either;
    use std::iter;
    Box::new(self.section_ids().flat_map(move |r| {
      match r {
        | Err(e) => Either::Left(iter::once(Err(e))),
        | Ok(id) => Either::Right(self.subsections_in(&id)),
      }
    }))
  }

  pub fn crates_in(
    &self, section_id: &str, subsection_id: &str,
  ) -> Box<dyn Iterator<Item = Result<String, CrateDirError>>> {
    let section_name = [self.prefix.to_string(), section_id.to_string()].concat();
    Box::new(CrateIterator::new(self.root.join(section_name).join(subsection_id), subsection_id))
  }

  pub fn crate_ids(&self) -> Box<dyn Iterator<Item = Result<String, CrateDirError>> + '_> {
    use either::Either;
    use std::iter;
    Box::new(self.subsection_ids().flat_map(move |r| {
      match r {
        | Err(e) => Either::Left(iter::once(Err(e))),
        | Ok(id) => {
          match self.crate_first(&id) {
            | Err(e) => Either::Left(iter::once(Err(e))),
            | Ok(first) => Either::Right(self.crates_in(&first, &id)),
          }
        },
      }
    }))
  }
}

impl InBandDirIterator {
  pub(crate) fn new<R>(root: R) -> InBandDirIterator
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
  pub(crate) fn new<R, P>(root: R, prefix: P) -> SectionIterator
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
  type Item = Result<String, CrateDirError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| CrateDirError::NotUnicode(entry_name.to_owned()))?;

      if let Some(c) = entry_str.strip_prefix(&self.prefix) {
        if c.len() == 1 {
          Ok(c.to_owned())
        } else {
          Err(CrateDirError::BadSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
        }
      } else {
        Err(CrateDirError::NonSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}

impl SubSectionIterator {
  pub(crate) fn new<R, P>(root: R, prefix: P) -> SubSectionIterator
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
  type Item = Result<String, CrateDirError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| CrateDirError::NotUnicode(entry_name.to_owned()))?;

      if 2 >= entry_str.len() && entry_str.starts_with(&self.prefix) {
        Ok(entry_str.to_owned())
      } else {
        Err(CrateDirError::BadSubSection(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}

impl CrateIterator {
  pub(crate) fn new<R, P>(root: R, prefix: P) -> CrateIterator
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
  type Item = Result<String, CrateDirError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|d| {
      let entry_name = d?.file_name();
      let entry_str = entry_name.to_str().ok_or_else(|| CrateDirError::NotUnicode(entry_name.to_owned()))?;

      if entry_str.starts_with(&self.prefix) {
        Ok(entry_str.to_owned())
      } else {
        Err(CrateDirError::BadCrate(entry_name, self.prefix.to_owned(), self.root.to_owned()))
      }
    })
  }
}
