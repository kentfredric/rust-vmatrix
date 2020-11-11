use std::{
  ffi::OsString,
  path::{Path, PathBuf},
};

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
        return Err(Error::BadSection(entry_name, root.as_ref().into()));
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
      return Err(Error::BadSubSection(entry_name, root.as_ref().into()));
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
      return Err(Error::BadCrate(entry_name, root.as_ref().into()));
    }
  }
  Ok(crates)
}

#[derive(Debug)]
pub enum Error {
  NotUnicode(OsString),
  IoError(std::io::Error),
  BadSection(OsString, PathBuf),
  BadSubSection(OsString, PathBuf),
  BadCrate(OsString, PathBuf),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
