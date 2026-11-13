#![no_main]

use bondline_io::sdf::{DataItemInput, SdfFile, SdfRecordInput, write_v2000};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(document) = SdfFile::parse(data) else {
        return;
    };
    assert_eq!(document.as_bytes(), data);
    assert!(data.is_ascii());
    assert!(data.len() <= 64 * 1024 * 1024);
    assert!(!document.records().is_empty());
    assert!(document.records().len() <= 1024);
    let mut previous_end = 0;
    let mut field_count = 0;
    for record in document.records() {
        let range = record.range();
        let mol = record.mol_range();
        let delimiter = record.delimiter_range();
        assert_eq!(range.start, previous_end);
        assert_eq!(mol.start, range.start);
        assert!(mol.end <= delimiter.start);
        assert_eq!(delimiter.end, range.end);
        assert!(data.get(delimiter.clone()).unwrap().starts_with(b"$$$$"));
        assert_eq!(data.get(mol.clone()).unwrap(), record.mol_bytes());
        previous_end = range.end;
        let mut field_end = mol.end;
        field_count += record.fields().len();
        for field in record.fields() {
            let item = field.range();
            let header = field.header_range();
            let value = field.value_range();
            assert!(field_end <= item.start);
            assert_eq!(item.start, header.start);
            assert_eq!(header.end, value.start);
            assert!(value.end < item.end);
            assert!(item.end <= delimiter.start);
            assert_eq!(data.get(value).unwrap(), field.raw_value());
            assert!(field.header().starts_with('>'));
            assert!(field.value_lines().iter().all(|line| line.len() <= 200));
            field_end = item.end;
        }

        let items = record
            .fields()
            .iter()
            .map(|field| DataItemInput {
                header: field.header(),
                value_lines: field.value_lines(),
            })
            .collect::<Vec<_>>();
        let output = match write_v2000(&[SdfRecordInput {
            mol_bytes: record.mol_bytes(),
            items: &items,
        }]) {
            Ok(bytes) => bytes,
            Err(error) => {
                assert_eq!(error.code(), "resource_limit");
                continue;
            }
        };
        let canonical = SdfFile::parse(&output).expect("successful SDF output must parse");
        assert_eq!(canonical.records().len(), 1);
        let recovered = canonical.records().first().unwrap();
        assert_eq!(recovered.mol_bytes(), record.mol_bytes());
        assert_eq!(recovered.fields().len(), record.fields().len());
        for (before, after) in record.fields().iter().zip(recovered.fields()) {
            assert_eq!(before.header(), after.header());
            assert_eq!(before.name(), after.name());
            assert_eq!(before.value_lines(), after.value_lines());
        }
    }
    assert_eq!(previous_end, data.len());
    assert!(field_count <= 65_536);
    assert_eq!(document.as_bytes(), data);
});
