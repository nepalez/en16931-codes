# en16931-codes

The code lists of [EN 16931] (the European standard on electronic invoicing) as published by the European Commission (version 17b, used from 2026-05-15).

<img src="https://cdn.evilmartians.com/badges/logo-no-label.svg" alt="Evil Martians logo" width="22" height="16" /> <b>en16931</b> is built by <b><a href="https://evilmartians.com/">Evil Martians</a></b>, an American design and engineering consultancy for <b>developer tools, AI, and cybersecurity startups</b>.

Every code list becomes a Rust `enum`. The enums are generated at compile time from the official code list file and embedded as `static` data. The crate makes no runtime allocations and performs no I/O.

The codes of the UN/EDIFACT data elements take their names from the UN/EDIFACT directory D.24A (`UNCL.24A`).

## Installation

Every code list stays behind a feature of its own, so a build contains only the lists it enables.

```toml
[dependencies]
en16931-codes = { version = "0.1", features = ["document-type", "tax-point-date"] }
```

## Code lists

The lists with one code per row give a single code per value. The lists whose codes differ in UBL and in CII give the codes of both syntaxes.

| Type | Feature | Kind | Description |
|---|---|---|---|
| `Country` | `country` | single | Country codes (ISO 3166-1 alpha-2) |
| `Currency` | `currency` | single | Currency codes (ISO 4217) |
| `IdentifierScheme` | `identifier-scheme` | single | Identifier schemes (ISO/IEC 6523 ICD) |
| `DocumentType` | `document-type` | single | Document types (UNTDID 1001) |
| `ReferenceQualifier` | `reference-qualifier` | single | Reference qualifiers (UNTDID 1153) |
| `VatIdentifierScheme` | `vat-identifier-scheme` | dual | VAT identifier schemes |
| `TaxRegistrationScheme` | `tax-registration-scheme` | single | Tax registration (fiscal) identifier schemes |
| `VatCategoryScheme` | `vat-category-scheme` | dual | VAT category schemes |
| `TaxPointDate` | `tax-point-date` | dual | VAT point date codes |
| `TextSubject` | `text-subject` | single | Text subject qualifiers (UNTDID 4451) |
| `PaymentMeans` | `payment-means` | single | Payment means (UNTDID 4461) |
| `TaxCategory` | `tax-category` | single | Duty, tax, and fee categories (UNTDID 5305) |
| `AllowanceReason` | `allowance-reason` | single | Allowance reasons (UNTDID 5189) |
| `ItemClassification` | `item-classification` | single | Item classification schemes (UNTDID 7143) |
| `ChargeReason` | `charge-reason` | single | Charge reasons (UNTDID 7161) |
| `MediaType` | `media-type` | single | MIME types of attached documents |
| `ElectronicAddressScheme` | `electronic-address-scheme` | single | Electronic address schemes (EAS) |
| `ExemptionReason` | `exemption-reason` | single | VAT exemption reasons (VATEX) |
| `Unit` | `unit` | single | Units of measure (UN/ECE Recommendations 20 and 21) |

## Other features

* `serde` serializes a code of a single list as its string and deserializes it back.
* `case-insensitive` adds `from_code_ignore_case` to the single lists and makes their parsing case-insensitive.

## Usage

### Single lists

```rust
use en16931_codes::{DocumentType, UnknownCode};

let invoice = DocumentType::from_code("380").expect("valid code");

assert_eq!(invoice, DocumentType::CommercialInvoice);
assert_eq!(invoice.code(), "380");
assert_eq!(invoice.name(), "Commercial invoice");
assert_eq!(invoice.remark(), Some("Invoice"));
assert_eq!(invoice.to_string(), "380");

assert_eq!("380".parse::<DocumentType>(), Ok(invoice));
assert_eq!("1".parse::<DocumentType>(), Err(UnknownCode("1".to_owned())));

assert!(DocumentType::ALL.contains(&invoice));
```

A single list provides:

* `code()`, `name()`, and `remark()` to read the code, its English name, and the remark of the code list file.
* `info()` to read all of them at once as a `CodeInfo`.
* `from_code()` to look up a code (case-sensitive).
* `from_code_ignore_case()` to look up a code ignoring the case (with the `case-insensitive` feature).
* `ALL` to iterate over every code in the order of the code list file.
* `Display`, `FromStr`, and `TryFrom<&str>` implementations.

### Dual lists

```rust
use en16931_codes::TaxPointDate;

let paid = TaxPointDate::from_cii_code("72").expect("valid code");

assert_eq!(paid, TaxPointDate::PaidToDate);
assert_eq!(paid.ubl_code(), "432");
assert_eq!(TaxPointDate::from_ubl_code("3").unwrap().cii_code(), "5");
```

A dual list provides:

* `ubl_code()` and `cii_code()` to read the code in either syntax.
* `name()` and `cii_name()` to read the English names in UBL and in CII.
* `info()` to read all of them at once as a `DualCodeInfo`.
* `from_ubl_code()` and `from_cii_code()` to look up a code in either syntax.
* `ALL` to iterate over every code in the order of the code list file.

## Crates

* `en16931-codes` provides the code list types.
* `en16931-codes-macros` provides the procedural macro that generates the types from the code list file.

## Development

The workspace uses [cargo-make] and [cargo-hack]. The `check` task verifies formatting and runs clippy and tests for every feature alone and for all features together.

```sh
cargo make check
```

The `generated-tests` feature adds a test for every generated code.

## Minimum supported Rust version

Rust 1.85 (edition 2024).

## License

The source code of both crates is licensed under the [MIT license].

The `en16931-codes` crate also contains data of the European Commission from the [EN 16931 code lists] file (version 17b). © European Union. The data is licensed under the [Creative Commons Attribution 4.0 International license][CC BY 4.0].

The crate includes the original file without changes. The generated types contain modified data. The codes are split into one type per code list, and their names are converted into Rust identifiers. The codes of the UN/EDIFACT data elements take their names from the UN/EDIFACT directory D.24A instead of the code list file.

The data is provided "as is", without warranties of any kind. See Section 5 of [CC BY 4.0] for the disclaimer of warranties.

### UN/CEFACT disclaimer

The names of the UN/EDIFACT codes come from the UN/EDIFACT directory D.24A, an output of UN/CEFACT.

> UN/CEFACT draws attention to the possibility that the practice or implementation of its outputs (which include but are not limited to Recommendations, norms, standards, guidelines and technical specifications) may involve the use of a claimed intellectual property right.
>
> Each output is based on the contributions of participants in the UN/CEFACT process, who have agreed to waive enforcement of their intellectual property rights pursuant to the UN/CEFACT IPR Policy (document ECE/TRADE/CEFACT/2006/11 available at <http://www.unece.org/cefact/> or from the UNECE secretariat). UN/CEFACT takes no position concerning the evidence, validity or applicability of any claimed intellectual property right or any other right that might be claimed by any third parties related to the implementation of its outputs. UN/CEFACT makes no representation that it has made any investigation or effort to evaluate any such rights.
>
> Implementers of UN/CEFACT outputs are cautioned that any third party intellectual property rights claims related to their use of a UN/CEFACT output will be their responsibility and are urged to ensure that their use of UN/CEFACT outputs does not infringe on an intellectual property right of a third party.
>
> UN/CEFACT does not accept any liability for any possible infringement of a claimed intellectual property right or any other right that might be claimed to relate to the implementation of any of its outputs.

[CC BY 4.0]: https://creativecommons.org/licenses/by/4.0/
[EN 16931]: https://ec.europa.eu/digital-building-blocks/sites/display/DIGITAL/Obtaining+a+copy+of+the+European+standard+on+eInvoicing
[EN 16931 code lists]: https://ec.europa.eu/digital-building-blocks/sites/display/DIGITAL/Registry+of+supporting+artefacts+to+implement+EN16931
[MIT license]: https://opensource.org/licenses/MIT
[cargo-hack]: https://github.com/taiki-e/cargo-hack
[cargo-make]: https://github.com/sagiegurari/cargo-make
