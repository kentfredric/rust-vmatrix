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
    let entry = entry_result?;
    let entry_name = entry.file_name();
    let entry_str = entry_name.to_str().ok_or_else(|| Error::NotUnicode(entry_name.to_owned()))?;
    if let Some(c) = entry_str.strip_prefix(prefix.as_ref()) {
      if c.len() == 1 && entry.file_type()?.is_dir() {
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
  let mut x = Vec::new();
  for subsection in std::fs::read_dir(root.as_ref())? {
    let subsection_entry = subsection?;
    let subsection_name = subsection_entry.file_name();
    let subsection_name_str =
      subsection_name.to_str().ok_or_else(|| Error::NotUnicode(subsection_name.to_owned()))?.to_owned();

    match subsection_name_str.len() {
      | 0 => continue,
      | 1 | 2 => {
        if subsection_name_str.starts_with(prefix.as_ref()) {
          x.push(subsection_name_str)
        } else {
          return Err(Error::BadSubSection(subsection_name, root.as_ref().into()));
        }
      },
      | _ => {
        return Err(Error::BadSubSection(subsection_name, root.as_ref().into()));
      },
    }
  }
  Ok(x)
}

pub(crate) fn crates_in<C, P>(root: P, prefix: C) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
  C: AsRef<str>,
{
  let mut x = Vec::new();
  for member in std::fs::read_dir(root.as_ref())? {
    let member_entry = member?;
    let member_name = member_entry.file_name();
    let member_name_str = member_name.to_str().ok_or_else(|| Error::NotUnicode(member_name.to_owned()))?.to_owned();

    if member_name_str.starts_with(prefix.as_ref()) {
      if member_entry.file_type()?.is_dir() {
        x.push(member_name_str)
      } else {
        return Err(Error::BadCrate(member_name, root.as_ref().into()));
      }
    } else {
      return Err(Error::BadCrate(member_name, root.as_ref().into()));
    }
  }
  Ok(x)
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
