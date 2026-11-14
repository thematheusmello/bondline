#![no_main]

use bondline_io::{
    mol::Version,
    sdf::{DataItemInput, SdfDocument, SdfFile, SdfRecordInput, write, write_v2000},
};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fuzz_target!(|data: &[u8]| {
    let Ok(document) = SdfFile::parse(data) else {
        return;
    };
    assert_eq!(document.as_bytes(), data);
    assert!(data.is_ascii());
    assert!(data.len() <= 64 * 1024 * 1024);
    assert!(!document.records().is_empty());
    assert!(document.records().len() <= 1024);
    let owned =
        SdfDocument::parse(Arc::from(data)).expect("accepted SDF must retain complete source");
    let cloned = owned.clone();
    assert_eq!(owned.as_bytes(), data);
    assert_eq!(owned, cloned);
    assert!(std::ptr::eq(owned.as_bytes(), cloned.as_bytes()));
    assert!(std::ptr::eq(owned.records(), cloned.records()));
    assert_eq!(owned.records().len(), document.records().len());
    assert_eq!(
        SdfDocument::retained_bytes([&owned, &owned]),
        SdfDocument::retained_bytes([&owned])
    );
    let view = owned
        .read()
        .expect("immutable owned source must remain valid");
    assert_eq!(view.as_bytes(), data);
    for (index, (source, retained)) in document.records().iter().zip(owned.records()).enumerate() {
        assert_eq!(retained.id().get() as usize, index + 1);
        assert_eq!(owned.record(retained.id()), Some(retained));
        assert_eq!(retained.range(), source.range());
        assert_eq!(retained.mol_range(), source.mol_range());
        assert_eq!(retained.delimiter_range(), source.delimiter_range());
        assert_eq!(retained.mol_version(), source.mol_version());
        assert_eq!(retained.field_count(), source.fields().len());
    }
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
        let inputs = [SdfRecordInput {
            mol_bytes: record.mol_bytes(),
            items: &items,
        }];
        let legacy = write_v2000(&inputs);
        if record.mol_version() == Version::V3000 {
            assert_eq!(legacy.as_ref().unwrap_err().code(), "unsupported_content");
        }
        let output = match write(&inputs) {
            Ok(bytes) => bytes,
            Err(error) => {
                assert_eq!(error.code(), "resource_limit");
                continue;
            }
        };
        if record.mol_version() == Version::V2000 {
            assert_eq!(legacy.unwrap(), output);
        }
        let canonical = SdfFile::parse(&output).expect("successful SDF output must parse");
        assert_eq!(canonical.records().len(), 1);
        let recovered = canonical.records().first().unwrap();
        assert_eq!(recovered.mol_bytes(), record.mol_bytes());
        assert_eq!(recovered.mol_version(), record.mol_version());
        assert_eq!(recovered.fields().len(), record.fields().len());
        for (before, after) in record.fields().iter().zip(recovered.fields()) {
            assert_eq!(before.header(), after.header());
            assert_eq!(before.name(), after.name());
            assert_eq!(before.dt_number(), after.dt_number());
            assert_eq!(before.value_lines(), after.value_lines());
        }
    }
    assert_eq!(previous_end, data.len());
    assert!(field_count <= 65_536);
    assert_eq!(document.as_bytes(), data);

    if let Some((index, record)) = view
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| !record.fields().is_empty())
    {
        let metadata = owned.records().get(index).unwrap();
        let field = record.fields().first().unwrap();
        let no_op = owned
            .with_field_value(metadata.id(), 0, field.value_lines())
            .expect("identical logical lines must preserve owned source");
        assert!(std::ptr::eq(owned.as_bytes(), no_op.as_bytes()));
        assert!(std::ptr::eq(owned.records(), no_op.records()));
        let value = ["bondline fuzz value"];
        let edited = match owned.with_field_value(metadata.id(), 0, &value) {
            Ok(edited) => edited,
            Err(error) => {
                assert_eq!(error.code(), "resource_limit");
                return;
            }
        };
        if field.value_lines() == value {
            assert_eq!(edited, owned);
        } else {
            let range = field.value_range();
            let mut expected = data.get(..range.start).unwrap().to_vec();
            expected.extend_from_slice(b"bondline fuzz value\n");
            expected.extend_from_slice(data.get(range.end..).unwrap());
            assert_eq!(edited.as_bytes(), expected);
        }
        assert_eq!(owned.as_bytes(), data);
        assert_eq!(owned.records().len(), edited.records().len());
        for (before, after) in owned.records().iter().zip(edited.records()) {
            assert_eq!(before.id(), after.id());
            assert_eq!(before.mol_version(), after.mol_version());
            assert_eq!(before.field_count(), after.field_count());
            assert_eq!(
                owned.as_bytes().get(before.mol_range()).unwrap(),
                edited.as_bytes().get(after.mol_range()).unwrap()
            );
        }
    }
});
