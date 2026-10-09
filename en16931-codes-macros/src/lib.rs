//! Proc-macro for generating the EN 16931 code list types
//! from the code list file the European Commission publishes for EN 16931.

use crate::prelude::{
    LitStr, PathBuf, Punctuated, Token, TokenStream, TokenStream2, Xlsx, format_ident,
    open_workbook, parse_macro_input, quote,
};
use crate::sheets::{Codes, Dual, List, SHEETS, Single, read};
use crate::uncl::names;

mod naming;
mod prelude;
mod sheets;
mod uncl;

// Writes an optional text as a token stream of an optional static string.
fn optional(value: &Option<String>) -> TokenStream2 {
    match value {
        Some(text) => quote! { Some(#text) },
        None => quote! { None },
    }
}

// Builds the name of the generated test of a code.
fn test_name(code: &str) -> proc_macro2::Ident {
    let code: String = code
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format_ident!("code_{code}")
}

// Checks that the manifest of the calling crate declares the feature of every code list.
fn check_features(manifest: &str) {
    let declared: Vec<_> = manifest
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, _)| name.trim())
        .collect();
    let missing: Vec<_> = SHEETS
        .iter()
        .map(|sheet| sheet.feature)
        .filter(|feature| !declared.contains(feature))
        .collect();
    assert!(
        missing.is_empty(),
        "the manifest declares no features for the code lists: {}",
        missing.join(", ")
    );
}

// Writes the documentation of the type of a code list.
fn type_doc(list: &List) -> String {
    match &list.terms {
        Some(terms) => format!(
            "{} (`{}` of EN 16931, used in {terms}).",
            list.title, list.sheet.tab
        ),
        None => format!("{} (`{}` of EN 16931).", list.title, list.sheet.tab),
    }
}

// Generates the type of a code list of one code per row.
fn single_list(list: &List, codes: &[Single]) -> TokenStream2 {
    let feature = list.sheet.feature;
    let ident = format_ident!("{}", list.sheet.type_name);
    let doc = type_doc(list);
    let count = codes.len();
    let variants: Vec<_> = codes
        .iter()
        .map(|c| format_ident!("{}", c.variant))
        .collect();
    let values: Vec<_> = codes.iter().map(|c| &c.code).collect();
    let names: Vec<_> = codes.iter().map(|c| &c.name).collect();
    let remarks: Vec<_> = codes.iter().map(|c| optional(&c.remark)).collect();
    let docs: Vec<_> = codes
        .iter()
        .map(|c| format!("{} (`{}`).", c.name, c.code))
        .collect();
    let tests_module = format_ident!("{}_generated_tests", list.sheet.feature.replace('-', "_"));
    let tests: Vec<_> = codes.iter().map(|c| test_name(&c.code)).collect();

    quote! {
        #[doc = #doc]
        #[cfg(feature = #feature)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum #ident {
            #(
                #[doc = #docs]
                #variants,
            )*
        }

        #[cfg(feature = #feature)]
        impl #ident {
            /// Returns all static metadata for this code.
            #[inline]
            #[must_use]
            pub const fn info(self) -> CodeInfo {
                match self {
                    #(
                        Self::#variants => CodeInfo {
                            code: #values,
                            name: #names,
                            remark: #remarks,
                        },
                    )*
                }
            }

            /// The code.
            #[inline]
            #[must_use]
            pub const fn code(self) -> &'static str {
                self.info().code
            }

            /// The English name.
            #[inline]
            #[must_use]
            pub const fn name(self) -> &'static str {
                self.info().name
            }

            /// The remark of the code list file on the code, if any.
            #[inline]
            #[must_use]
            pub const fn remark(self) -> Option<&'static str> {
                self.info().remark
            }

            /// Look up a code by its string representation (case-sensitive).
            #[inline]
            #[must_use]
            pub fn from_code(code: &str) -> Option<Self> {
                match code {
                    #( #values => Some(Self::#variants), )*
                    _ => None,
                }
            }

            /// Look up a code by its string representation (case-insensitive).
            #[inline]
            #[cfg(feature = "case-insensitive")]
            #[must_use]
            pub fn from_code_ignore_case(code: &str) -> Option<Self> {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|value| value.code().eq_ignore_ascii_case(code))
            }

            /// Every code in the order of the code list file.
            pub const ALL: &'static [Self; #count] = &[
                #( Self::#variants, )*
            ];
        }

        #[cfg(feature = #feature)]
        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.code())
            }
        }

        #[cfg(feature = #feature)]
        impl core::str::FromStr for #ident {
            type Err = UnknownCode;

            fn from_str(s: &str) -> Result<Self, UnknownCode> {
                #[cfg(feature = "case-insensitive")]
                let result = Self::from_code_ignore_case(s);

                #[cfg(not(feature = "case-insensitive"))]
                let result = Self::from_code(s);

                result.ok_or_else(|| UnknownCode(s.to_owned()))
            }
        }

        #[cfg(feature = #feature)]
        impl<'a> TryFrom<&'a str> for #ident {
            type Error = UnknownCode;

            fn try_from(s: &'a str) -> Result<Self, UnknownCode> {
                s.parse()
            }
        }

        #[cfg(all(feature = #feature, feature = "serde"))]
        impl crate::prelude::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: crate::prelude::Serializer,
            {
                serializer.serialize_str(self.code())
            }
        }

        #[cfg(all(feature = #feature, feature = "serde"))]
        impl<'de> crate::prelude::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: crate::prelude::Deserializer<'de>,
            {
                let code = <&str as crate::prelude::Deserialize>::deserialize(deserializer)?;
                code.parse().map_err(<D::Error as crate::prelude::DeError>::custom)
            }
        }

        #[cfg(all(test, feature = #feature, feature = "generated-tests"))]
        mod #tests_module {
            use super::*;
            #(
                #[test]
                fn #tests() {
                    let value: #ident = #values.parse().unwrap();
                    assert_eq!(value, #ident::#variants);
                    assert_eq!(value.code(), #values);
                    assert_eq!(value.to_string(), #values);
                    assert_eq!(#ident::from_code(#values), Some(value));
                }
            )*
        }
    }
}

// Generates the type of a code list of a code of UBL and a code of CII per row.
fn dual_list(list: &List, codes: &[Dual]) -> TokenStream2 {
    let feature = list.sheet.feature;
    let ident = format_ident!("{}", list.sheet.type_name);
    let doc = type_doc(list);
    let count = codes.len();
    let variants: Vec<_> = codes
        .iter()
        .map(|c| format_ident!("{}", c.variant))
        .collect();
    let ubl_codes: Vec<_> = codes.iter().map(|c| &c.ubl_code).collect();
    let ubl_names: Vec<_> = codes.iter().map(|c| &c.ubl_name).collect();
    let cii_codes: Vec<_> = codes.iter().map(|c| &c.cii_code).collect();
    let cii_names: Vec<_> = codes.iter().map(|c| &c.cii_name).collect();
    let docs: Vec<_> = codes
        .iter()
        .map(|c| {
            format!(
                "{} (`{}` in UBL, `{}` in CII).",
                c.ubl_name, c.ubl_code, c.cii_code
            )
        })
        .collect();
    let tests_module = format_ident!("{}_generated_tests", list.sheet.feature.replace('-', "_"));
    let tests: Vec<_> = codes.iter().map(|c| test_name(&c.ubl_code)).collect();

    quote! {
        #[doc = #doc]
        #[cfg(feature = #feature)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum #ident {
            #(
                #[doc = #docs]
                #variants,
            )*
        }

        #[cfg(feature = #feature)]
        impl #ident {
            /// Returns all static metadata for this code.
            #[inline]
            #[must_use]
            pub const fn info(self) -> DualCodeInfo {
                match self {
                    #(
                        Self::#variants => DualCodeInfo {
                            ubl_code: #ubl_codes,
                            ubl_name: #ubl_names,
                            cii_code: #cii_codes,
                            cii_name: #cii_names,
                        },
                    )*
                }
            }

            /// The code in UBL.
            #[inline]
            #[must_use]
            pub const fn ubl_code(self) -> &'static str {
                self.info().ubl_code
            }

            /// The code in CII.
            #[inline]
            #[must_use]
            pub const fn cii_code(self) -> &'static str {
                self.info().cii_code
            }

            /// The English name of the code in UBL.
            #[inline]
            #[must_use]
            pub const fn name(self) -> &'static str {
                self.info().ubl_name
            }

            /// The English name of the code in CII.
            #[inline]
            #[must_use]
            pub const fn cii_name(self) -> &'static str {
                self.info().cii_name
            }

            /// Look up a code by its representation in UBL (case-sensitive).
            #[inline]
            #[must_use]
            pub fn from_ubl_code(code: &str) -> Option<Self> {
                match code {
                    #( #ubl_codes => Some(Self::#variants), )*
                    _ => None,
                }
            }

            /// Look up a code by its representation in CII (case-sensitive).
            #[inline]
            #[must_use]
            pub fn from_cii_code(code: &str) -> Option<Self> {
                match code {
                    #( #cii_codes => Some(Self::#variants), )*
                    _ => None,
                }
            }

            /// Every code in the order of the code list file.
            pub const ALL: &'static [Self; #count] = &[
                #( Self::#variants, )*
            ];
        }

        #[cfg(all(test, feature = #feature, feature = "generated-tests"))]
        mod #tests_module {
            use super::*;
            #(
                #[test]
                fn #tests() {
                    let value = #ident::from_ubl_code(#ubl_codes).unwrap();
                    assert_eq!(value, #ident::#variants);
                    assert_eq!(value.ubl_code(), #ubl_codes);
                    assert_eq!(value.cii_code(), #cii_codes);
                    assert_eq!(#ident::from_cii_code(#cii_codes), Some(value));
                }
            )*
        }
    }
}

/// Generates a type for every code list of the code list file the European Commission
/// publishes for EN 16931.
///
/// The macro takes two paths relative to the manifest directory of the calling crate:
/// the code list file of the European Commission, and the code list file (UNCL)
/// of the UN/EDIFACT directory that names the codes of the UN/EDIFACT data elements.
/// Every type stays behind a feature named after its code list,
/// and the manifest of the calling crate must declare all of them.
/// The generated code refers to `serde` through `crate::prelude`,
/// so the calling crate re-exports `Serialize`, `Serializer`, `Deserialize`,
/// `Deserializer`, and `DeError` there when it enables its `serde` feature.
///
/// Usage: `en16931_codes!("EN16931 code lists values v17b - used from 2026-05-15.xlsx", "UNCL.24A");`
#[proc_macro]
pub fn en16931_codes(input: TokenStream) -> TokenStream {
    let paths = parse_macro_input!(input with Punctuated::<LitStr, Token![,]>::parse_terminated);
    let [lists_path, directory_path] = <[_; 2]>::try_from(
        paths.iter().map(LitStr::value).collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| panic!("the macro takes two paths: the code lists and the directory"));
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_dir = PathBuf::from(manifest_dir);
    let manifest = std::fs::read_to_string(manifest_dir.join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("cannot read the manifest of the calling crate: {e}"));
    check_features(&manifest);

    let directory_path = manifest_dir.join(directory_path);
    let directory = std::fs::read_to_string(&directory_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", directory_path.display()));
    let directory = names(&directory);

    let path = manifest_dir.join(lists_path);
    let mut workbook: Xlsx<_> =
        open_workbook(&path).unwrap_or_else(|e| panic!("cannot open {}: {e}", path.display()));
    let lists: Vec<_> = read(&mut workbook, &directory)
        .iter()
        .map(|list| match &list.codes {
            Codes::Single(codes) => single_list(list, codes),
            Codes::Dual(codes) => dual_list(list, codes),
        })
        .collect();

    quote! {
        /// All metadata of a code of a list of one code per syntax.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct CodeInfo {
            /// The code.
            pub code: &'static str,
            /// The English name of the code.
            pub name: &'static str,
            /// The remark of the code list file on the code, if any: the interpretation
            /// of a document type, the usage of a payment means, the semantic model of
            /// a tax category, the remark on an exemption reason, or the source of a unit.
            pub remark: Option<&'static str>,
        }

        /// All metadata of a code of a list whose codes differ in UBL and in CII.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct DualCodeInfo {
            /// The code in UBL.
            pub ubl_code: &'static str,
            /// The English name of the code in UBL.
            pub ubl_name: &'static str,
            /// The code in CII.
            pub cii_code: &'static str,
            /// The English name of the code in CII.
            pub cii_name: &'static str,
        }

        /// Error returned when parsing an unrecognised EN 16931 code.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct UnknownCode(pub String);

        impl core::fmt::Display for UnknownCode {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "unknown EN 16931 code: {:?}", self.0)
            }
        }

        impl std::error::Error for UnknownCode {}

        #( #lists )*
    }
    .into()
}
