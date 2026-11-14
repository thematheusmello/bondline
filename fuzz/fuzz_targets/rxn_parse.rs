#![no_main]
use bondline_io::{
    mol::Version,
    rxn::{ComponentInput, MAX_BYTES, MAX_COMPONENTS, Role, RxnFile, write_v2000},
};
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Ok(file) = RxnFile::parse(data) {
        assert_eq!(file.as_bytes(), data);
        assert_eq!(file.container_version(), Version::V2000);
        assert!(data.len() <= MAX_BYTES && file.components().len() <= MAX_COMPONENTS);
        let mut previous = file.counts_range().end;
        let mut counts = [0_usize; 3];
        let mut rank = 0;
        for component in file.components() {
            let next = match component.role() {
                Role::Reactant => 0,
                Role::Product => 1,
                Role::Reagent => 2,
            };
            assert!(next >= rank);
            rank = next;
            counts[next] += 1;
            assert_eq!(component.range().start, previous);
            assert_eq!(
                data.get(component.mol_range()).unwrap(),
                component.mol_bytes()
            );
            assert!(
                data.get(component.range()).unwrap().starts_with(b"$MOL\n")
                    || data
                        .get(component.range())
                        .unwrap()
                        .starts_with(b"$MOL\r\n")
            );
            previous = component.range().end;
        }
        assert_eq!(previous, data.len());
        assert_eq!(
            counts,
            [
                file.counts().reactants,
                file.counts().products,
                file.counts().reagents
            ]
        );
        let inputs = file
            .components()
            .iter()
            .map(|component| ComponentInput {
                role: component.role(),
                mol_bytes: component.mol_bytes(),
            })
            .collect::<Vec<_>>();
        match write_v2000(file.header(), &inputs) {
            Ok(encoded) => {
                assert!(encoded.len() <= MAX_BYTES);
                let after = RxnFile::parse(&encoded).unwrap();
                assert_eq!(after.header(), file.header());
                assert_eq!(after.counts(), file.counts());
                assert_eq!(after.components().len(), file.components().len());
                for (before, after) in file.components().iter().zip(after.components()) {
                    assert_eq!(before.role(), after.role());
                    assert_eq!(before.version(), after.version());
                    if before.mol_bytes().ends_with(b"\n") {
                        assert_eq!(before.mol_bytes(), after.mol_bytes());
                    } else {
                        assert_eq!([before.mol_bytes(), b"\n"].concat(), after.mol_bytes());
                    }
                }
            }
            Err(error) => assert_eq!(error.code(), "resource_limit"),
        }
        assert_eq!(file.as_bytes(), data);
    }
});
