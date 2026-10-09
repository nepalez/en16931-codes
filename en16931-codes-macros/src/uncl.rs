//! The reader of the names of codes from the code list file (UNCL) of a UN/EDIFACT directory.

use crate::prelude::HashMap;

// The change indicators a line may start with: none, addition, change of an entry,
// change of a name, change of a text, and deletion.
const INDICATORS: &str = " +*#|X";

// The change indicator of a code marked for deletion.
const DELETED: char = 'X';

// The column where the code of an entry starts (one more in a few misaligned lines).
const CODE_COLUMNS: [usize; 2] = [5, 6];

// The column where a wrapped name of a code continues.
const NAME_COLUMN: usize = 11;

// The first column of the description of a code.
const DESCRIPTION_COLUMN: usize = 14;

/// The names of the active codes of the data elements, by the number of an element.
pub(crate) type Names = HashMap<String, HashMap<String, String>>;

// The part of an element the previous line belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Metadata,
    Name,
    CodeDescription,
}

// A code as read from the file, before the deleted codes are dropped.
struct Entry {
    code: String,
    name: String,
    deleted: bool,
}

/// Reads the names of the active codes of every data element of the file.
pub(crate) fn names(text: &str) -> Names {
    let mut elements: Vec<(String, Vec<Entry>)> = Vec::new();
    let mut section = None;
    let lines = text
        .lines()
        .enumerate()
        .skip_while(|(_, line)| !line.starts_with("-----"));

    for (index, line) in lines {
        let number = index + 1;
        if line.trim().is_empty() || line.starts_with("-----") {
            section = None;
            continue;
        }
        let indicator = line.chars().next().unwrap_or(' ');
        assert!(
            INDICATORS.contains(indicator),
            "line {number}: unknown change indicator in {line:?}"
        );
        let rest = &line[indicator.len_utf8()..];
        let body = rest.trim_start();
        let column = 1 + rest.len() - body.len();

        if let Some(element) = header(body, column) {
            elements.push((element, Vec::new()));
            section = None;
            continue;
        }
        let entries = &mut elements
            .last_mut()
            .unwrap_or_else(|| panic!("line {number}: {line:?} precedes any data element"))
            .1;

        if body.starts_with("Note:") && !entries.is_empty() {
            section = Some(Section::CodeDescription);
        } else if body.starts_with("Desc:")
            || body.starts_with("Repr:")
            || body.starts_with("Note:")
        {
            section = Some(Section::Metadata);
        } else if entries.is_empty()
            && section == Some(Section::Metadata)
            && column > CODE_COLUMNS[1]
        {
        } else if CODE_COLUMNS.contains(&column) {
            let (code, name) = body
                .split_once("  ")
                .unwrap_or_else(|| panic!("line {number}: the code {body:?} has no name"));
            entries.push(Entry {
                code: code.to_owned(),
                name: name.trim().to_owned(),
                deleted: indicator == DELETED,
            });
            section = Some(Section::Name);
        } else if column == NAME_COLUMN && section == Some(Section::Name) {
            let entry = entries.last_mut().expect("a code precedes its name");
            if !entry.name.ends_with('-') {
                entry.name.push(' ');
            }
            entry.name.push_str(body);
        } else if column >= DESCRIPTION_COLUMN
            && matches!(section, Some(Section::Name | Section::CodeDescription))
        {
            section = Some(Section::CodeDescription);
        } else {
            panic!("line {number}: unexpected layout of {line:?}");
        }
    }

    elements
        .into_iter()
        .map(|(element, entries)| {
            let names = entries
                .into_iter()
                .filter(|entry| !entry.deleted)
                .map(|entry| (entry.code, entry.name))
                .collect();
            (element, names)
        })
        .collect()
}

// Reads the number of a data element from its header.
fn header(body: &str, column: usize) -> Option<String> {
    if !(4..=5).contains(&column) {
        return None;
    }
    let (number, rest) = body.split_at_checked(4)?;
    if !number.chars().all(|c| c.is_ascii_digit()) || !rest.starts_with("  ") {
        return None;
    }
    rest.trim_end()
        .strip_suffix(']')?
        .trim_end_matches(['B', 'I', 'C'])
        .strip_suffix('[')?;
    Some(number.to_owned())
}
