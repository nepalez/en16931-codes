//! The sheets of the code list file and the reader of their codes.

use crate::naming::variants;
use crate::prelude::{BufReader, Data, File, HashMap, HashSet, Range, Reader, Xlsx};
use crate::uncl::Names;

// The sheet that names every code list and the business terms that use it.
const INDEX: &str = "Index";

// The columns of the index: the name of a code list, its sheet, and its business terms.
const INDEX_TITLE: usize = 0;
const INDEX_SHEET: usize = 1;
const INDEX_TERMS: usize = 6;

/// The layout of the codes in a sheet.
pub(crate) enum Layout {
    /// One code per row, below a single header row.
    ///
    /// The codes of a UN/EDIFACT data element take their names from the directory.
    Single {
        code: usize,
        name: Option<usize>,
        remark: Option<usize>,
        element: Option<&'static str>,
    },
    /// A code of UBL and a code of CII per row, below the header row of the codes.
    Dual,
}

/// A sheet of the code list file with the type and the feature it gives.
pub(crate) struct Sheet {
    pub tab: &'static str,
    pub type_name: &'static str,
    pub feature: &'static str,
    pub layout: Layout,
}

/// Every sheet of the code list file except the index.
pub(crate) const SHEETS: [Sheet; 19] = [
    single("Country", "Country", "country", 1, Some(0), None),
    single("Currency", "Currency", "currency", 1, Some(0), None),
    single(
        "ICD",
        "IdentifierScheme",
        "identifier-scheme",
        0,
        Some(1),
        None,
    ),
    untdid("1001", "DocumentType", "document-type", "1001", Some(2)),
    untdid(
        "1153",
        "ReferenceQualifier",
        "reference-qualifier",
        "1153",
        None,
    ),
    dual("VAT ID", "VatIdentifierScheme", "vat-identifier-scheme"),
    single(
        "FISCAL ID",
        "TaxRegistrationScheme",
        "tax-registration-scheme",
        0,
        Some(1),
        None,
    ),
    dual("VAT CAT", "VatCategoryScheme", "vat-category-scheme"),
    dual("Time", "TaxPointDate", "tax-point-date"),
    untdid("Text", "TextSubject", "text-subject", "4451", None),
    untdid("Payment", "PaymentMeans", "payment-means", "4461", Some(2)),
    untdid("5305", "TaxCategory", "tax-category", "5305", Some(2)),
    untdid(
        "Allowance",
        "AllowanceReason",
        "allowance-reason",
        "5189",
        None,
    ),
    untdid(
        "Item",
        "ItemClassification",
        "item-classification",
        "7143",
        None,
    ),
    untdid("Charge", "ChargeReason", "charge-reason", "7161", None),
    single("MIME", "MediaType", "media-type", 0, None, None),
    single(
        "EAS",
        "ElectronicAddressScheme",
        "electronic-address-scheme",
        0,
        Some(1),
        None,
    ),
    single(
        "VATEX",
        "ExemptionReason",
        "exemption-reason",
        0,
        Some(1),
        Some(2),
    ),
    single("Unit", "Unit", "unit", 1, Some(2), Some(0)),
];

// Describes a sheet of one code per row.
const fn single(
    tab: &'static str,
    type_name: &'static str,
    feature: &'static str,
    code: usize,
    name: Option<usize>,
    remark: Option<usize>,
) -> Sheet {
    Sheet {
        tab,
        type_name,
        feature,
        layout: Layout::Single {
            code,
            name,
            remark,
            element: None,
        },
    }
}

// Describes a sheet of the codes of a UN/EDIFACT data element, one code per row.
const fn untdid(
    tab: &'static str,
    type_name: &'static str,
    feature: &'static str,
    element: &'static str,
    remark: Option<usize>,
) -> Sheet {
    Sheet {
        tab,
        type_name,
        feature,
        layout: Layout::Single {
            code: 0,
            name: Some(1),
            remark,
            element: Some(element),
        },
    }
}

// Describes a sheet of a code of UBL and a code of CII per row.
const fn dual(tab: &'static str, type_name: &'static str, feature: &'static str) -> Sheet {
    Sheet {
        tab,
        type_name,
        feature,
        layout: Layout::Dual,
    }
}

/// A code list read from its sheet.
pub(crate) struct List {
    pub sheet: &'static Sheet,
    pub title: String,
    pub terms: Option<String>,
    pub codes: Codes,
}

/// The codes of a list in the order of the sheet.
pub(crate) enum Codes {
    Single(Vec<Single>),
    Dual(Vec<Dual>),
}

/// A code of a list of one code per row.
pub(crate) struct Single {
    pub variant: String,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
}

/// A code of a list of a code of UBL and a code of CII per row.
pub(crate) struct Dual {
    pub variant: String,
    pub ubl_code: String,
    pub ubl_name: String,
    pub cii_code: String,
    pub cii_name: String,
}

/// Reads every code list of the code list file, naming the codes of UN/EDIFACT data
/// elements after the directory.
pub(crate) fn read(workbook: &mut Xlsx<BufReader<File>>, directory: &Names) -> Vec<List> {
    let index = range(workbook, INDEX);
    SHEETS
        .iter()
        .map(|sheet| {
            let row = index
                .rows()
                .find(|row| text(row, INDEX_SHEET).as_deref() == Some(sheet.tab))
                .unwrap_or_else(|| panic!("the index names no sheet {:?}", sheet.tab));
            let range = range(workbook, sheet.tab);
            List {
                sheet,
                title: text(row, INDEX_TITLE).unwrap_or_default(),
                terms: text(row, INDEX_TERMS),
                codes: match sheet.layout {
                    Layout::Single {
                        code,
                        name,
                        remark,
                        element,
                    } => {
                        let names = element.map(|element| {
                            directory.get(element).unwrap_or_else(|| {
                                panic!("the directory has no data element {element}")
                            })
                        });
                        Codes::Single(singles(sheet.tab, &range, code, name, remark, names))
                    }
                    Layout::Dual => Codes::Dual(duals(sheet.tab, &range)),
                },
            }
        })
        .collect()
}

// Reads a sheet of the code list file.
fn range(workbook: &mut Xlsx<BufReader<File>>, tab: &str) -> Range<Data> {
    workbook
        .worksheet_range(tab)
        .unwrap_or_else(|e| panic!("cannot read the sheet {tab:?}: {e}"))
}

// Reads a cell as a trimmed text with collapsed whitespace, or a whole number as digits.
fn text(row: &[Data], column: usize) -> Option<String> {
    let text = match row.get(column)? {
        Data::String(s) => s.split_whitespace().collect::<Vec<_>>().join(" "),
        Data::Float(f) if f.fract() == 0.0 => format!("{f:.0}"),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        _ => String::new(),
    };
    (!text.is_empty()).then_some(text)
}

// Reads the codes of a sheet of one code per row below a single header row.
// The names of the directory, when given, replace the names of the sheet.
fn singles(
    tab: &str,
    range: &Range<Data>,
    code: usize,
    name: Option<usize>,
    remark: Option<usize>,
    directory: Option<&HashMap<String, String>>,
) -> Vec<Single> {
    let rows: Vec<_> = range
        .rows()
        .skip(1)
        .filter_map(|row| {
            let code = text(row, code)?;
            let name = match directory {
                Some(names) => names.get(&code).cloned().unwrap_or_else(|| {
                    panic!("the directory has no code {code:?} of the sheet {tab:?}")
                }),
                None => name.map_or_else(|| Some(code.clone()), |column| text(row, column))?,
            };
            Some((code, name, remark.and_then(|column| text(row, column))))
        })
        .collect();
    let pairs: Vec<_> = rows
        .iter()
        .map(|(c, n, _)| (c.clone(), n.clone()))
        .collect();
    unique(tab, pairs.iter().map(|(code, _)| code));
    rows.into_iter()
        .zip(variants(tab, &pairs))
        .map(|((code, name, remark), variant)| Single {
            variant,
            code,
            name,
            remark,
        })
        .collect()
}

// Reads the codes of a sheet of a code of UBL and a code of CII per row
// below the header row of the codes.
fn duals(tab: &str, range: &Range<Data>) -> Vec<Dual> {
    let rows: Vec<_> = range
        .rows()
        .skip_while(|row| {
            !text(row, 0).is_some_and(|label| label == "Code" || label.ends_with(" Code"))
        })
        .skip(1)
        .filter_map(|row| Some((text(row, 0)?, text(row, 1)?, text(row, 2)?, text(row, 3)?)))
        .collect();
    assert!(!rows.is_empty(), "the sheet {tab:?} has no codes");
    let pairs: Vec<_> = rows
        .iter()
        .map(|(c, n, _, _)| (c.clone(), n.clone()))
        .collect();
    unique(tab, rows.iter().map(|(code, ..)| code));
    unique(tab, rows.iter().map(|(_, _, code, _)| code));
    rows.into_iter()
        .zip(variants(tab, &pairs))
        .map(|((ubl_code, ubl_name, cii_code, cii_name), variant)| Dual {
            variant,
            ubl_code,
            ubl_name,
            cii_code,
            cii_name,
        })
        .collect()
}

// Checks that no code repeats in a sheet.
fn unique<'a>(tab: &str, codes: impl Iterator<Item = &'a String>) {
    let mut seen = HashSet::new();
    for code in codes {
        assert!(
            seen.insert(code),
            "the code {code:?} repeats in the sheet {tab:?}"
        );
    }
}
