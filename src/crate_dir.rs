use std::{
  ffi::OsString,
  fs, io,
  path::{Path, PathBuf},
};

/// Errors from CrateDir handling
#[derive(thiserror::Error, Debug)]
pub enum CrateDirError {
  /// A Non-Unicode [`OsString`]
  ///
  /// Trying to decode an [`OsString`] as some [`str`] variant had
  /// problems due to [`OsString`] being malformed as per Unicode
  /// requirements.
  #[error("Could not decode {0:?} as Unicode")]
  NotUnicode(OsString),

  /// A directory traversal induced [`std::io::Error`]
  ///
  /// These are rare, but can happen if you hit a race between
  /// determining a directory entry is itself, a directory, and the
  /// resulting `opendir` call.
  #[error("IO Error in stats repo directory: {0}")]
  IoError(#[from] std::io::Error),

  /// Invalid section suffix for matching section prefix
  ///
  /// A directory entry in a `root` was discovered including the
  /// required `prefix`, but had characters after the prefix that
  /// made it violate the designated layout scheme.
  ///
  /// Eg:
  /// * `<ROOT>/crates-a` -> `Ok('a')`
  /// * `<ROOT>/crates-ax` ->
  ///   `Err(BadSection('crates-ax','a',PathBuf::from('/some/path'
  ///   )))`
  #[error(
    "Directory Section {0:?} in {2:?} does not satisfy the layout scheme (should be one character after prefix {1})"
  )]
  BadSection(OsString, String, PathBuf),

  /// Non-Prefix matching entry in root directory
  ///
  /// A directory entry in a `root` was discovered, but it didn't
  /// have the required `prefix`, and so is not viable for
  /// automatic traversal. Seeing this error in your [`Iterator`]
  /// result can usually just be ignored, but could help in
  /// debugging situations.
  #[error("Directory Section {0:?} in {2:?} does not satisfy the layout scheme (doesn't start with prefix {1})")]
  NonSection(OsString, String, PathBuf),

  /// Invalid subsection for a given section
  ///
  /// A directory entry in a `path` was discovered where it was
  /// required to start with a given `prefix` character, and isn't
  /// the required length
  ///
  /// These usually indicate misplaced subsections or cruft files
  /// inside the directory tree, and can be either ignored or
  /// panicked on based on user taste.
  ///
  /// But due to violating the naming constraints, it won't be
  /// automatically traversed.
  ///
  /// Eg:
  /// * `<ROOT>/crates-a/bx` : Errors as `b` is not valid in
  ///   `crates-a/`
  /// * `<ROOT>/crates-a/aay` : Errors as `aay` is too long, and
  ///   should be at most `aa`
  /// * `<ROOT>/crates-a/a` : Fine
  /// * `<ROOT>/crates-a/ab` : Fine
  #[error(
    "Subsection {0:?} in {2:?} does not satisfy the layout scheme (should be 1-or-2 characters and start with {1})"
  )]
  BadSubSection(OsString, String, PathBuf),

  /// Invalid Crate within crate subsection
  ///
  /// A directory entry in a `path` was discovered, representing a
  /// `crate`, but it didn't have the required `prefix` letter
  /// pair.
  ///
  /// Usually, this indicates a `crate` directory is placed in the
  /// wrong `subsection`, or other filesystem cruft. Iterator only
  /// return `Err()` for these and not traverse. Consumer can do
  /// whatever they want with it, but ignoring it is fine.
  ///
  /// Eg:
  /// * `<ROOT>/crates-a/ab/abc` : Fine
  /// * `<ROOT>/crates-a/ab/aac` : Error, should be in `crates-a/aa/`
  /// * `<ROOT>/crates-a/ab/bcc` : Error, should be in `crates-b/bc/`
  /// * `<ROOT>/crates-a/a/a`    : Should be fine...
  /// * `<ROOT>/crates-a/a/ab`  : Error, should be in `crates-a/ab`
  #[error("Crate {0:?} in {2:?} does not satisfy the layout scheme (should start with {1})")]
  BadCrate(OsString, String, PathBuf),

  /// Invalid name for crate
  ///
  /// As far as I know this can only happen if somehow, the consumer
  /// screwed up and asked for a crate directory by passing `&""`
  /// as an "expected crate name". This doesn't work as directory
  /// names can't have 0 length.
  #[error("Crate name {0:?} is illegal, must have at least one character")]
  BadCrateName(String),
}

#[derive(Debug)]
struct InBandDirIterator {
  root:  PathBuf,
  inner: Option<Result<fs::ReadDir, ()>>,
}

#[derive(Debug)]
struct SectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
struct SubSectionIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

#[derive(Debug)]
struct CrateIterator {
  root:   PathBuf,
  prefix: String,
  inner:  InBandDirIterator,
}

/// Utility crate for mapping/discovering crate directory paths
///
/// A Crate Directory layout is a 3 level structure designed to
/// minimise the tax on filesystem caches and so-forth, by
/// fragmenting names into sections.
///
/// The topology is somewhat inspired by the CPAN "AuthorID" scheme,
/// where `AUTHORFOO`'s files are stashed in `A/AU/AUTHORFOO/`
///
/// ```text
/// <ROOT>
///     /<PREFIX><CRATENAME[0]>   # section
///         /<CRATENAME[0..1]>    # subsection
///             /<CRATENAME>      # crate
///                 /<FILES>      # crate files
/// ```
///
/// This struct provides facilities for efficient discovery of items
/// that exist, as well as query based "I assume this exists, where
/// is it" approaches.
///
/// # Iterator Model
/// Iterators herein wrap [`std::fs::ReadDir`], but with a novel
/// twist: Instead of returning a `Result<Iterator<Item =
/// Result<_,E>>,E>`, making iterator chaining with error handling
/// somewhat contorted, the underlying errors are returned **inside**
/// the iteration stream, allowing the errors to be piped down to the
/// consumer as much as possible, allowing consumers to handle errors
/// mid-iteration more organically, including just ignoring
/// them if it suits them.
///
/// This gives a simpler `Iterator<Item=Result<_,E>>` interface, at
/// the small cost of making handling the actual error in `Err()`
/// cases slightly more odd in some cases.
///
/// At first glance to most people this seems weird, but it makes
/// sense if your goal is to produce a single stream of results, like
/// the command `find` would return, while also efficiently doing
/// depth-first iteration, and applying rules at each level, ending
/// up under the hood basically having...
/// ```text
///     Iterator(root) -> [
///         dir-a -> Iterator("dir-a") -> [
///             aa -> Iterator("dir-a/aa") -> [
///               aaa
///               aab
///               aac
///             ],
///             ... 60 more iterators here ...
///         ]
///         ... 60 more iterators here ...
///     ]
/// ```
#[derive(Debug)]
pub struct CrateDir {
  root:   PathBuf,
  prefix: String,
}

impl CrateDir {
  /// Construct a [`CrateDir`] for traversing `root` using `prefix`
  /// to build the 'base' segment
  ///
  /// # Example
  /// ```
  /// # use std::path::Path;
  /// # use vmatrix::CrateDir;
  /// let c = CrateDir::new(Path::new("/tmp/crateinfo-root"), "crates-");
  /// ```
  pub fn new(root: &Path, prefix: &str) -> Self { CrateDir { root: root.to_owned(), prefix: prefix.to_owned() } }

  fn crate_first(&self, crate_name: &str) -> Result<String, CrateDirError> {
    crate_name.chars().next().ok_or_else(|| CrateDirError::BadCrateName(crate_name.to_owned())).map(Into::into)
  }

  fn section_name(&self, crate_name: &str) -> Result<String, CrateDirError> {
    self.crate_first(crate_name).map(|v| [self.prefix.to_string(), v].concat())
  }

  fn subsection_name(&self, crate_name: &str) -> Result<String, CrateDirError> {
    self.crate_first(crate_name).map(|first| {
      match crate_name.chars().nth(1) {
        | None => first,
        | Some(c) => [first, c.to_string()].concat(),
      }
    })
  }

  fn path_to(&self, crate_name: &str) -> Result<PathBuf, CrateDirError> {
    self.section_name(crate_name).and_then(|sname| {
      self.subsection_name(crate_name).map(|ssname| PathBuf::from(sname).join(ssname).join(crate_name))
    })
  }

  fn path_to_file(&self, crate_name: &str, file_name: &str) -> Result<PathBuf, CrateDirError> {
    self.path_to(crate_name).map(|x| x.join(file_name))
  }

  pub(crate) fn abs_path_to(&self, crate_name: &str) -> Result<PathBuf, CrateDirError> {
    self.path_to(crate_name).map(|x| self.root.join(x))
  }

  pub(crate) fn abs_path_to_file(&self, crate_name: &str, file_name: &str) -> Result<PathBuf, CrateDirError> {
    self.abs_path_to(crate_name).map(|x| x.join(file_name))
  }

  /// Returns an iterator of crate section id's without `prefix`
  ///
  /// # Example
  /// ```no_run
  /// # use std::path::Path;
  /// # use vmatrix::{CrateDir,CrateDirError};
  /// # fn main() -> Result<(),CrateDirError> {
  /// for id_res in CrateDir::new(Path::new("/tmp/crateinfo-root"), "crates-").section_ids() {
  ///   match id_res {
  ///     // Lists suffix part of directory entries only, eg: "a"
  ///     | Ok(id) => println!("Section ID: {}", id),
  ///     // Lists traversal errors and irrelevant entries
  ///     | Err(e) => eprintln!("Error: {}", e),
  ///   }
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn section_ids(&self) -> impl Iterator<Item = Result<String, CrateDirError>> {
    SectionIterator::new(&self.root, &self.prefix)
  }

  /// Returns an iterator of crate section names with `prefix`
  ///
  /// # Example
  /// ```no_run
  /// # use std::path::Path;
  /// # use vmatrix::{CrateDir,CrateDirError};
  /// # fn main() -> Result<(),CrateDirError> {
  /// for name_res in CrateDir::new(Path::new("/tmp/crateinfo-root"), "crates-").section_names() {
  ///   match name_res {
  ///     // Lists legal directory entries only, eg, "crates-a"
  ///     | Ok(name) => println!("Section: {}", name),
  ///     // Lists traversal errors and irrelevant entries
  ///     | Err(e) => eprintln!("Error: {}", e),
  ///   }
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn section_names(&self) -> impl Iterator<Item = Result<String, CrateDirError>> {
    let prefix = self.prefix.to_owned();
    self.section_ids().map(move |r| r.map(|s| [prefix.to_string(), s].concat()))
  }

  fn subsections_in(&self, section_id: &str) -> impl Iterator<Item = Result<String, CrateDirError>> {
    let section_name = [self.prefix.to_string(), section_id.to_string()].concat();
    SubSectionIterator::new(&self.root.join(section_name), section_id)
  }

  /// Returns an iterator of second level subsection ids
  ///
  /// # Example
  /// ```no_run
  /// # use std::path::Path;
  /// # use vmatrix::{CrateDir,CrateDirError};
  /// # fn main() -> Result<(),CrateDirError> {
  /// for id_res in CrateDir::new(Path::new("/tmp/crateinfo-root"), "crates-").subsection_ids() {
  ///   match id_res {
  ///     // Lists legal second level directory entries only, eg, "a", "aa", "ab" ... "ba",
  ///     | Ok(id) => println!("SubSection: {}", id),
  ///     // Lists traversal errors and irrelevant entries
  ///     | Err(e) => eprintln!("Error: {}", e),
  ///   }
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn subsection_ids(&self) -> impl Iterator<Item = Result<String, CrateDirError>> + '_ {
    use either::Either;
    use std::iter;
    self.section_ids().flat_map(move |r| {
      match r {
        | Err(e) => Either::Left(iter::once(Err(e))),
        | Ok(id) => Either::Right(self.subsections_in(&id)),
      }
    })
  }

  fn crates_in(&self, section_id: &str, subsection_id: &str) -> impl Iterator<Item = Result<String, CrateDirError>> {
    let section_name = [self.prefix.to_string(), section_id.to_string()].concat();
    CrateIterator::new(&self.root.join(section_name).join(subsection_id), subsection_id)
  }

  /// Returns an iterator of crate identifiers
  ///
  /// # Example
  /// ```no_run
  /// # use std::path::Path;
  /// # use vmatrix::{CrateDir,CrateDirError};
  /// # fn main() -> Result<(),CrateDirError> {
  /// for id_res in CrateDir::new(Path::new("/tmp/crateinfo-root"), "crates-").crate_ids() {
  ///   match id_res {
  ///     // Lists legal 3rd level directory entries only, eg, "cargo","clippy",...,"nom"
  ///     | Ok(id) => println!("SubSection: {}", id),
  ///     // Lists traversal errors and irrelevant entries
  ///     | Err(e) => eprintln!("Error: {}", e),
  ///   }
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn crate_ids(&self) -> impl Iterator<Item = Result<String, CrateDirError>> + '_ {
    use either::Either;
    use std::iter;
    self.subsection_ids().flat_map(move |r| {
      match r {
        | Err(e) => Either::Left(iter::once(Err(e))),
        | Ok(id) => {
          match self.crate_first(&id) {
            | Err(e) => Either::Left(iter::once(Err(e))),
            | Ok(first) => Either::Right(self.crates_in(&first, &id)),
          }
        },
      }
    })
  }
}

impl InBandDirIterator {
  fn new(root: &Path) -> InBandDirIterator { InBandDirIterator { root: root.to_path_buf(), inner: None } }
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
  fn new(root: &Path, prefix: &str) -> SectionIterator {
    SectionIterator { root: root.to_path_buf(), prefix: prefix.to_string(), inner: InBandDirIterator::new(root) }
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
  fn new(root: &Path, prefix: &str) -> SubSectionIterator {
    SubSectionIterator { root: root.to_path_buf(), prefix: prefix.to_string(), inner: InBandDirIterator::new(root) }
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
  fn new(root: &Path, prefix: &str) -> CrateIterator {
    CrateIterator { root: root.to_path_buf(), prefix: prefix.to_string(), inner: InBandDirIterator::new(root) }
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
