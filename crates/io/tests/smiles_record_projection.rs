//! Independent whole-source, live-ID and immutable-edit projection expectations.

use bondline_io::{
    StoredDocument,
    smiles::{RecordSelection, SmilesFileDocument},
};
use bondline_model::{PageSize, Point2};
use bondline_render::Cancellation;
use std::sync::Arc;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &[u8] = b"CC first\r\nC1.N.C1 \t\0\xffPRIVATE-META\r\nCC duplicate\tname\n";

#[test]
fn all_record_projection_keeps_the_complete_source_without_faking_full_mapping() -> TestResult {
    let source = SmilesFileDocument::parse(Arc::from(SOURCE))?;
    let original = source.clone();
    let stored = StoredDocument::from_smiles_records(
        source,
        RecordSelection::All,
        48.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &Cancellation::default(),
    )?;
    assert!(stored.has_smiles_projection());
    assert!(!stored.is_fully_mapped());
    assert_eq!(stored.model().pages().len(), 1);
    assert_eq!(stored.model().pages()[0].fragments().len(), 3);
    assert_eq!(stored.smiles_file()?.as_bytes(), SOURCE);
    assert_eq!(
        stored.smiles_file()?.as_bytes().as_ptr(),
        original.as_bytes().as_ptr()
    );
    assert_eq!(stored.smiles_records().ok_or("record mappings")?.len(), 3);
    assert_eq!(original.as_bytes(), SOURCE);
    Ok(())
}

// Expanded expectations preserve the initial1,313-byte missing-API declaration.
use bondline_io::{DocumentError, smiles::file::RecordId};
use bondline_model::{AtomId, BondOrder, Fragment, FragmentId, PageId};
use serde_json::{Value, json};
use std::{
    io::Write as _,
    num::NonZeroU16,
    process::{Command, Stdio},
};

const EDITED: &[u8] = b"CC first\r\n[CH2]=[CH2].[NH3] \t\0\xffPRIVATE-META\r\nCC duplicate\tname\n";

fn record(id: u32) -> TestResult<RecordId> {
    RecordId::new(id).ok_or_else(|| "positive record".into())
}
fn page() -> TestResult<PageId> {
    PageId::new(1).ok_or_else(|| "page1".into())
}
fn owner(bytes: &[u8]) -> TestResult<SmilesFileDocument> {
    Ok(SmilesFileDocument::parse(Arc::from(bytes))?)
}
fn project(
    source: SmilesFileDocument,
    selection: RecordSelection,
) -> Result<StoredDocument, DocumentError> {
    StoredDocument::from_smiles_records(
        source,
        selection,
        48.0,
        Point2::new(72.0, 72.0).map_err(|_| DocumentError::UnsupportedEdit)?,
        PageSize::A4,
        &Cancellation::default(),
    )
}
fn whole() -> TestResult<StoredDocument> {
    Ok(project(owner(SOURCE)?, RecordSelection::All)?)
}

fn double_middle(stored: &StoredDocument) -> TestResult<Fragment> {
    let middle = &stored.model().pages()[0].fragments()[1];
    let mut atoms = middle.atoms().to_vec();
    atoms[0].explicit_hydrogens = Some(2);
    atoms[2].explicit_hydrogens = Some(2);
    let mut bonds = middle.bonds().to_vec();
    bonds[0].order = BondOrder::Double;
    Ok(Fragment::new(middle.id(), atoms, bonds)?)
}

fn consume(script: &str, args: &[String], input: &[u8]) -> TestResult<Value> {
    let mut child = Command::new(std::env::var_os("BONDLINE_PYTHON").ok_or("pinned Python")?)
        .arg("-B")
        .arg(format!("{}/tests/{script}", env!("CARGO_MANIFEST_DIR")))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("consumer stdin")?
        .write_all(input)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn raw(stored: &StoredDocument, edited: bool) -> TestResult {
    let result = consume(
        "smiles_file_raw_oracle.py",
        &[],
        stored.smiles_file()?.as_bytes(),
    )?;
    assert_eq!(result["byte_length"], if edited { 63 } else { 53 });
    assert_eq!(
        result["sha256"],
        if edited {
            "8f505ce1309d75cf50bdda5ec4ffd4513119bd0b290aae7484992e8ff52990f5"
        } else {
            "598f6b118ec2bff73da8aaee9dbd29a1fb54a50dce81865139aabd94194f0804"
        }
    );
    assert_eq!(result["physical_lines"], 3);
    assert_eq!(
        result["records"][0],
        json!({"line_number":1,"full_range":[0,10],"notation_range":[0,2],"separator_range":[2,3],"application_data_range":[3,8],"line_ending_range":[8,10],"notation_hex":"4343","application_data_hex":"6669727374"})
    );
    let delta = if edited { 10 } else { 0 };
    assert_eq!(result["records"][1]["full_range"], json!([10, 35 + delta]));
    assert_eq!(
        result["records"][1]["notation_range"],
        json!([10, 17 + delta])
    );
    assert_eq!(
        result["records"][1]["separator_range"],
        json!([17 + delta, 19 + delta])
    );
    assert_eq!(
        result["records"][1]["application_data_range"],
        json!([19 + delta, 33 + delta])
    );
    assert_eq!(
        result["records"][1]["application_data_hex"],
        "00ff505249564154452d4d455441"
    );
    assert_eq!(
        result["records"][1]["line_ending_range"],
        json!([33 + delta, 35 + delta])
    );
    assert_eq!(
        result["records"][2],
        json!({"line_number":3,"full_range":[35+delta,53+delta],"notation_range":[35+delta,37+delta],"separator_range":[37+delta,38+delta],"application_data_range":[38+delta,52+delta],"line_ending_range":[52+delta,53+delta],"notation_hex":"4343","application_data_hex":"6475706c6963617465096e616d65"})
    );
    Ok(())
}

fn middle_chemistry(stored: &StoredDocument, edited: bool) -> TestResult {
    let fragment = &stored.model().pages()[0].fragments()[1];
    let h = if edited { 2 } else { 3 };
    let order = if edited { 2 } else { 1 };
    let options = json!({"atoms":[[6,null,h],[7,null,3],[6,null,h]],"bonds":[[1,3,order]],"formula":if edited {"C2H7N"} else {"C2H9N"},"components":[[1,3],[2]]});
    let result = consume(
        "smiles_authoring_oracle.py",
        &[
            if edited {
                "C1.N.C=1".into()
            } else {
                "C1.N.C1".into()
            },
            options.to_string(),
        ],
        &serde_json::to_vec(fragment.as_ref())?,
    )?;
    assert_eq!(result["raw_graph"]["atoms"], options["atoms"]);
    assert_eq!(result["raw_graph"]["bonds"], options["bonds"]);
    assert_eq!(result["no_assigned_stereo"], true);
    assert_eq!(result["no_radicals"], true);
    Ok(())
}

#[test]
fn literal_closure_maps_and_changed_dfs_token_order_keep_ids_directions_and_opaque_bytes()
-> TestResult {
    let stored = whole()?;
    let original = stored.clone();
    let maps = stored.smiles_records().ok_or("maps")?;
    assert_eq!(
        maps.iter()
            .map(|m| (m.record_id().get(), m.fragment_id().get()))
            .collect::<Vec<_>>(),
        [(1, 1), (2, 2), (3, 3)]
    );
    let map = &maps[1];
    assert_eq!(
        map.atoms()
            .iter()
            .map(|a| (a.atom_id.get(), a.notation_atom, a.range.clone()))
            .collect::<Vec<_>>(),
        [(3, 0, 10..11), (4, 1, 13..14), (5, 2, 15..16)]
    );
    let bond = &map.bonds()[0];
    assert_eq!(
        (
            bond.bond_id.get(),
            bond.notation_bond,
            bond.atoms.map(AtomId::get)
        ),
        (2, 0, [3, 5])
    );
    assert_eq!(bond.bond_range, None);
    let ring = bond.ring.as_ref().ok_or("literal ring closure")?;
    assert_eq!(ring.label, 1);
    assert_eq!(ring.open_range, 11..12);
    assert_eq!(ring.close_range, 16..17);
    assert_eq!(ring.open_bond_range, None);
    assert_eq!(ring.close_bond_range, None);
    assert_eq!(
        maps[2]
            .atoms()
            .iter()
            .map(|a| a.range.clone())
            .collect::<Vec<_>>(),
        [35..36, 36..37]
    );
    raw(&stored, false)?;
    middle_chemistry(&stored, false)?;
    let replacement = double_middle(&stored)?;
    let edited =
        stored.with_smiles_fragment(page()?, replacement.clone(), &Cancellation::default())?;
    assert_eq!(stored, original);
    assert_eq!(stored.smiles_file()?.as_bytes(), SOURCE);
    assert_eq!(edited.smiles_file()?.as_bytes(), EDITED);
    assert!(!edited.is_fully_mapped());
    assert!(edited.has_smiles_projection());
    assert_eq!(edited.model().pages()[0].size(), PageSize::A4);
    assert_eq!(
        edited.model().pages()[0].fragments()[1].as_ref(),
        &replacement
    );
    for at in [0, 2] {
        assert!(Arc::ptr_eq(
            &stored.model().pages()[0].fragments()[at],
            &edited.model().pages()[0].fragments()[at]
        ));
    }
    let map = &edited.smiles_records().ok_or("edited maps")?[1];
    assert_eq!(
        map.atoms()
            .iter()
            .map(|a| (a.atom_id.get(), a.notation_atom, a.range.clone()))
            .collect::<Vec<_>>(),
        [(3, 0, 10..15), (5, 1, 16..21), (4, 2, 22..27)]
    );
    let bond = &map.bonds()[0];
    assert_eq!(
        (
            bond.bond_id.get(),
            bond.notation_bond,
            bond.atoms.map(AtomId::get)
        ),
        (2, 0, [3, 5])
    );
    assert_eq!(bond.bond_range, Some(15..16));
    assert_eq!(bond.ring, None);
    assert_eq!(
        edited.smiles_records().ok_or("maps")?[2]
            .atoms()
            .iter()
            .map(|a| a.range.clone())
            .collect::<Vec<_>>(),
        [45..46, 46..47]
    );
    let source = edited.source_info().ok_or("source info")?;
    assert_eq!(
        (source.format, source.byte_length, source.modified),
        ("smiles", 53, true)
    );
    raw(&edited, true)?;
    middle_chemistry(&edited, true)?;
    assert!(!format!("{edited:?}").contains("PRIVATE-META"));
    Ok(())
}

#[test]
fn exact_noop_shares_model_source_index_and_mapping_without_rewriting_spelling() -> TestResult {
    let stored = whole()?;
    let same = stored.with_smiles_fragment(
        page()?,
        stored.model().pages()[0].fragments()[1].as_ref().clone(),
        &Cancellation::default(),
    )?;
    assert_eq!(same, stored);
    assert!(Arc::ptr_eq(
        &same.model_snapshot(),
        &stored.model_snapshot()
    ));
    assert_eq!(
        same.smiles_file()?.as_bytes().as_ptr(),
        stored.smiles_file()?.as_bytes().as_ptr()
    );
    assert_eq!(
        same.smiles_file()?.records().as_ptr(),
        stored.smiles_file()?.records().as_ptr()
    );
    assert_eq!(
        same.smiles_records().ok_or("maps")?.as_ptr(),
        stored.smiles_records().ok_or("maps")?.as_ptr()
    );
    assert_eq!(same.smiles_file()?.as_bytes(), SOURCE);
    assert!(!same.source_info().ok_or("source")?.modified);
    assert_eq!(
        StoredDocument::retained_bytes([&stored, &same]),
        StoredDocument::retained_bytes([&stored]) + std::mem::size_of::<StoredDocument>()
    );
    Ok(())
}

#[test]
fn selected_record_retains_unprojectable_unselected_source_but_all_refuses_atomically() -> TestResult
{
    let bytes = b"CC chosen\r\ns1cccc1 unsupported\t\0\xff\n";
    let source = owner(bytes)?;
    let before = source.clone();
    let selected = project(source.clone(), RecordSelection::Record(record(1)?))?;
    assert_eq!(selected.model().pages()[0].fragments().len(), 1);
    assert_eq!(
        selected.smiles_records().ok_or("map")?[0].record_id().get(),
        1
    );
    assert_eq!(selected.smiles_file()?.as_bytes(), bytes);
    assert_eq!(
        selected.smiles_file()?.as_bytes().as_ptr(),
        source.as_bytes().as_ptr()
    );
    assert_eq!(
        project(source.clone(), RecordSelection::All)
            .err()
            .ok_or("later unsupported record")?
            .code(),
        "unsupported_content"
    );
    assert_eq!(source, before);
    assert_eq!(source.as_bytes(), bytes);
    assert_eq!(
        project(source, RecordSelection::Record(record(3)?))
            .err()
            .ok_or("missing positive record")?
            .code(),
        "not_found"
    );
    let empty = owner(b" \xffignored\r\n\n")?;
    assert_eq!(
        project(empty, RecordSelection::All)
            .err()
            .ok_or("empty selection")?
            .code(),
        "unsupported_content"
    );
    Ok(())
}

#[test]
fn unmapped_source_display_is_qualified_but_detached_geometry_cannot_render_or_save() -> TestResult
{
    let stored = whole()?;
    let scene = bondline_io::display::drawing_page_scene(
        1024,
        1024,
        &stored,
        page()?,
        &Cancellation::default(),
    )?;
    assert!(!scene.scene().primitives.is_empty());
    assert_eq!(
        stored
            .to_cdxml()
            .err()
            .ok_or("opaque source must not become CDXML Save")?
            .code(),
        "unsupported_content"
    );
    let mut detached = stored.clone();
    detached
        .model_mut()
        .set_page_size(page()?, PageSize::new(612.0, 792.0)?)?;
    assert!(detached.smiles_file().is_err());
    assert!(
        bondline_io::display::drawing_page_scene(
            1024,
            1024,
            &detached,
            page()?,
            &Cancellation::default()
        )
        .is_err()
    );
    assert!(
        detached
            .with_smiles_fragment(
                page()?,
                stored.model().pages()[0].fragments()[1].as_ref().clone(),
                &Cancellation::default()
            )
            .is_err()
    );
    assert_eq!(stored.smiles_file()?.as_bytes(), SOURCE);
    Ok(())
}

#[test]
fn replacements_preserve_geometry_identity_direction_and_non_h_atom_fields() -> TestResult {
    let stored = whole()?;
    let middle = &stored.model().pages()[0].fragments()[1];
    let mut moved = middle.atoms().to_vec();
    let point = moved[0].position.ok_or("point")?;
    moved[0].position = Some(Point2::new(point.x_pt() + 1.0, point.y_pt())?);
    let mut isotope = middle.atoms().to_vec();
    isotope[0].isotope = NonZeroU16::new(13);
    let mut reordered = middle.atoms().to_vec();
    reordered.swap(0, 2);
    let mut reversed = middle.bonds().to_vec();
    let bond = &mut reversed[0];
    std::mem::swap(&mut bond.begin, &mut bond.end);
    for candidate in [
        Fragment::new(middle.id(), moved, middle.bonds().to_vec())?,
        Fragment::new(middle.id(), isotope, middle.bonds().to_vec())?,
        Fragment::new(middle.id(), reordered, middle.bonds().to_vec())?,
        Fragment::new(middle.id(), middle.atoms().to_vec(), reversed)?,
        Fragment::new(
            FragmentId::new(99).ok_or("id")?,
            middle.atoms().to_vec(),
            middle.bonds().to_vec(),
        )?,
    ] {
        assert!(
            stored
                .with_smiles_fragment(page()?, candidate, &Cancellation::default())
                .is_err()
        );
    }
    assert_eq!(stored, whole()?);
    assert_eq!(stored.smiles_file()?.as_bytes(), SOURCE);
    Ok(())
}

fn dense_129_bonds() -> String {
    let mut closures = vec![(0_usize, 63_usize)];
    for at in 0..64 {
        let other = (at + 2) % 64;
        closures.push((at.min(other), at.max(other)));
    }
    closures.push((0, 3));
    let mut markers = vec![String::new(); 64];
    for (index, (a, b)) in closures.into_iter().enumerate() {
        let n = index + 1;
        let marker = if n < 10 {
            n.to_string()
        } else {
            format!("%{n}")
        };
        markers[a].push_str(&marker);
        markers[b].push_str(&marker);
    }
    markers
        .into_iter()
        .map(|labels| format!("C{labels}"))
        .collect()
}

#[test]
fn whole_selected_caps_and_precancel_are_checked_before_layout_or_partial_candidates() -> TestResult
{
    let sixty_four = owner("C\n".repeat(64).as_bytes())?;
    let cancel = Cancellation::default();
    let at_limit = StoredDocument::from_smiles_records(
        sixty_four,
        RecordSelection::All,
        80.0,
        Point2::new(72.0, 72.0)?,
        PageSize::A4,
        &cancel,
    )?;
    assert_eq!(at_limit.model().pages()[0].fragments().len(), 64);
    assert_eq!(at_limit.smiles_file()?.records().len(), 64);
    for bytes in ["C\n".repeat(65), format!("{}\n", dense_129_bonds())] {
        let source = owner(bytes.as_bytes())?;
        let before = source.clone();
        assert_eq!(
            project(source.clone(), RecordSelection::All)
                .err()
                .ok_or("whole aggregate cap")?
                .code(),
            "resource_limit"
        );
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert_eq!(
            StoredDocument::from_smiles_records(
                source.clone(),
                RecordSelection::All,
                48.0,
                Point2::new(72.0, 72.0)?,
                PageSize::A4,
                &cancelled
            )
            .err()
            .ok_or("cancel")?
            .code(),
            "cancelled"
        );
        assert_eq!(source, before);
    }
    let stored = whole()?;
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        stored
            .with_smiles_fragment(page()?, double_middle(&stored)?, &cancelled)
            .err()
            .ok_or("cancelled splice")?
            .code(),
        "cancelled"
    );
    assert_eq!(stored.smiles_file()?.as_bytes(), SOURCE);
    assert_eq!(
        StoredDocument::from_smiles_records(
            owner(SOURCE)?,
            RecordSelection::All,
            0.0,
            Point2::new(72.0, 72.0)?,
            PageSize::A4,
            &cancel
        )
        .err()
        .ok_or("invalid length")?
        .code(),
        "invalid_params"
    );
    Ok(())
}
