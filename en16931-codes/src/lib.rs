//! The code lists of EN 16931 as published by the European Commission
//! (a version 17b, used from 2026-05-15).
//!
//! The data is generated at compile time from the official code list file,
//! and embedded as `static` data. No runtime allocations, no I/O.
//!
//! The codes of the UN/EDIFACT data elements take their names from the code list file
//! of the UN/EDIFACT directory D.24A (`UNCL.24A`), the list the code list file refers to.
//!
//! Every code list gives a type of its own behind a feature named after the list,
//! so a build contains only the code lists it enables.
//! The lists whose codes differ in UBL and in CII give the codes of both syntaxes.
//!
//! # Example
//!
//! ```rust
//! # #[cfg(all(feature = "document-type", feature = "tax-point-date"))]
//! # {
//! use en16931_codes::{DocumentType, TaxPointDate};
//!
//! let value = DocumentType::from_code("380").unwrap();
//! assert_eq!(value, DocumentType::CommercialInvoice);
//! assert_eq!(value.remark(), Some("Invoice"));
//!
//! let issue = TaxPointDate::from_ubl_code("3").unwrap();
//! assert_eq!(issue.cii_code(), "5");
//! # }
//! ```
//!
//! # License
//!
//! The source code is licensed under the [MIT license].
//!
//! The crate also contains data of the European Commission
//! from the [EN 16931 code lists] file (version 17b). © European Union. The data is licensed
//! under the [Creative Commons Attribution 4.0 International license][CC BY 4.0].
//!
//! The crate includes the original file without changes.
//! The generated types contain modified data. The codes are split into one type per code list,
//! and their names are converted into Rust identifiers. The codes of the UN/EDIFACT data elements
//! take their names from the UN/EDIFACT directory D.24A instead of the code list file.
//!
//! The data is provided "as is", without warranties of any kind. See Section 5 of [CC BY 4.0]
//! for the disclaimer of warranties.
//!
//! ## UN/CEFACT disclaimer
//!
//! The names of the UN/EDIFACT codes come from the UN/EDIFACT directory D.24A,
//! an output of UN/CEFACT.
//!
//! > UN/CEFACT draws attention to the possibility that the practice or implementation of its
//! > outputs (which include but are not limited to Recommendations, norms, standards, guidelines
//! > and technical specifications) may involve the use of a claimed intellectual property right.
//! >
//! > Each output is based on the contributions of participants in the UN/CEFACT process,
//! > who have agreed to waive enforcement of their intellectual property rights
//! > pursuant to the UN/CEFACT IPR Policy (document ECE/TRADE/CEFACT/2006/11 available at
//! > <http://www.unece.org/cefact/> or from the UNECE secretariat).
//! > UN/CEFACT takes no position concerning the evidence, validity or applicability of any
//! > claimed intellectual property right or any other right that might be claimed
//! > by any third parties related to the implementation of its outputs.
//! > UN/CEFACT makes no representation that it has made any investigation
//! > or effort to evaluate any such rights.
//! >
//! > Implementers of UN/CEFACT outputs are cautioned that any third party
//! > intellectual property rights claims related to their use of a UN/CEFACT output
//! > will be their responsibility and are urged to ensure that their use of UN/CEFACT outputs
//! > does not infringe on an intellectual property right of a third party.
//! >
//! > UN/CEFACT does not accept any liability for any possible infringement
//! > of a claimed intellectual property right or any other right that might be claimed
//! > to relate to the implementation of any of its outputs.
//!
//! [CC BY 4.0]: https://creativecommons.org/licenses/by/4.0/
//! [EN 16931 code lists]: https://ec.europa.eu/digital-building-blocks/sites/display/DIGITAL/Registry+of+supporting+artefacts+to+implement+EN16931
//! [MIT license]: https://opensource.org/licenses/MIT

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::too_many_lines, clippy::match_same_arms)]

mod prelude;

en16931_codes_macros::en16931_codes!(
    "EN16931 code lists values v17b - used from 2026-05-15.xlsx",
    "UNCL.24A"
);

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "document-type")]
    #[test]
    fn looks_up_a_code_with_its_remark() {
        let invoice = DocumentType::from_code("380").unwrap();

        assert_eq!(invoice, DocumentType::CommercialInvoice);
        assert_eq!(invoice.name(), "Commercial invoice");
        assert_eq!(invoice.remark(), Some("Invoice"));
    }

    #[cfg(feature = "tax-category")]
    #[test]
    fn names_a_code_of_a_data_element_after_the_directory() {
        let outside = TaxCategory::from_code("O").unwrap();

        assert_eq!(outside, TaxCategory::ServicesOutsideScopeOfTax);
        assert_eq!(outside.name(), "Services outside scope of tax");
    }

    #[cfg(feature = "country")]
    #[test]
    fn names_a_code_of_another_list_after_the_sheet() {
        let andorra = Country::from_code("AD").unwrap();

        assert_eq!(andorra.name(), "Andorra");
    }

    #[cfg(feature = "tax-point-date")]
    #[test]
    fn converts_a_code_between_the_syntaxes() {
        let paid = TaxPointDate::from_cii_code("72").unwrap();

        assert_eq!(paid, TaxPointDate::PaidToDate);
        assert_eq!(paid.ubl_code(), "432");
        assert_eq!(TaxPointDate::from_ubl_code("3").unwrap().cii_code(), "5");
    }

    #[cfg(feature = "vat-identifier-scheme")]
    #[test]
    fn keeps_the_codes_of_both_syntaxes() {
        let vat = VatIdentifierScheme::from_ubl_code("VAT").unwrap();

        assert_eq!(vat.cii_code(), "VA");
        assert_eq!(VatIdentifierScheme::from_cii_code("VAT"), None);
    }

    #[cfg(feature = "media-type")]
    #[test]
    fn names_a_code_without_a_name_after_the_code() {
        let pdf = MediaType::from_code("application/pdf").unwrap();

        assert_eq!(pdf, MediaType::ApplicationPdf);
        assert_eq!(pdf.name(), "application/pdf");
    }

    #[cfg(feature = "unit")]
    #[test]
    fn keeps_the_source_of_a_unit() {
        assert_eq!(Unit::from_code("KGM").unwrap().remark(), Some("rec20"));
        assert_eq!(Unit::from_code("XBX").unwrap().remark(), Some("rec21"));
    }

    #[cfg(feature = "document-type")]
    #[test]
    fn rejects_an_unknown_code() {
        assert_eq!(
            "1".parse::<DocumentType>(),
            Err(UnknownCode("1".to_owned()))
        );
    }

    #[cfg(all(feature = "currency", feature = "case-insensitive"))]
    #[test]
    fn looks_up_a_code_ignoring_the_case() {
        assert_eq!(
            Currency::from_code_ignore_case("eur"),
            Currency::from_code("EUR")
        );
    }

    #[cfg(all(feature = "document-type", feature = "serde"))]
    #[test]
    fn serializes_a_code_as_its_string() {
        let invoice = DocumentType::CommercialInvoice;

        let encoded = serde_json::to_string(&invoice).unwrap();
        let decoded: DocumentType = serde_json::from_str(&encoded).unwrap();

        assert_eq!(encoded, "\"380\"");
        assert_eq!(decoded, invoice);
    }
}
