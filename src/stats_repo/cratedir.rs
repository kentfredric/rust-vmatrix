use std::{
  ffi::OsString,
  path::{Path, PathBuf},
};

pub(crate) fn sections_in<P, C>(root: P, prefix: C) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
  C: AsRef<str>,
{
  let mut x = Vec::new();
  for section in std::fs::read_dir(root.as_ref())? {
    let section_entry = section?;
    let section_name = section_entry.file_name();
    let section_name_str = section_name.to_str().ok_or_else(|| Error::NotUnicode(section_name.to_owned()))?.to_owned();
    match section_name_str.strip_prefix(prefix.as_ref()) {
      | None => continue,
      | Some(c) => {
        match c.len() {
          | 1 => {
            if section_entry.file_type()?.is_dir() {
              x.push(c.to_owned());
            } else {
              return Err(Error::SectionNotDir(section_name));
            }
          },
          | _ => {
            return Err(Error::SectionNameInvalid(section_name));
          },
        }
      },
    }
  }
  Ok(x)
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
  SectionNotDir(OsString),
  SectionNameInvalid(OsString),
  IoError(std::io::Error),
  BadSubSection(OsString, PathBuf),
  BadCrate(OsString, PathBuf),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
