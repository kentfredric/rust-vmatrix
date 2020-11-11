use std::{ffi::OsString, path::Path};

pub(crate) fn sections_in_root<P>(root: P) -> Result<Vec<String>, self::Error>
where
  P: AsRef<Path>,
{
  let mut x = Vec::new();
  for section in std::fs::read_dir(root.as_ref())? {
    let section_entry = section?;
    let section_name = section_entry.file_name();
    let section_name_str = section_name.to_str().ok_or_else(|| Error::NotUnicode(section_name.to_owned()))?.to_owned();
    match section_name_str.strip_prefix("crates-") {
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

#[derive(Debug)]
pub enum Error {
  NotUnicode(OsString),
  SectionNotDir(OsString),
  SectionNameInvalid(OsString),
  IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
