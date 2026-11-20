//! Compare the shared Analysis command with molecules read independently by RDKit.
use super::{Fixture, Format};
use bondline_commands::{AnalysisOutcome, AnalysisScope, Command, Engine};
use bondline_io::{StoredDocument, mol::MolTransform, sdf::SdfFile};
use bondline_model::{PageSize, Point2};
use serde::Deserialize;
use serde_json::json;
use std::{error::Error, io::Write, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Reference {
    formula: String,
    atoms: usize,
}

#[derive(Default)]
pub(super) struct Counts {
    pub matched: usize,
    pub unavailable: usize,
}

#[derive(Default)]
pub(super) struct Coverage {
    pub files: Counts,
    pub records: Counts,
}

fn analysis(engine: &mut Engine) -> Result<AnalysisOutcome> {
    let params = serde_json::to_value(engine.analysis_request())?;
    Ok(engine
        .dispatch("analysis.formula", params)?
        .analysis
        .ok_or("missing Analysis result")?
        .outcome)
}

enum Comparison {
    Matched(String),
    Unavailable(String),
}

// These exact sources have passed the whole-drawing comparison independently.
const REQUIRED_DRAWINGS: &[&str] = &[
    "fadcbe8eee438dfb55232cb2c3baca41ea6613edb0d6163af9486b6c84ff6fc9", // RDKit butane
    "afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d", // RDKit methyl radical
    "1867a5904e742030431b523a31167951ff998007a89b9c3d94ec88cf7d4c4380", // V3000 ethane
];

fn require_proven_formula(file: &Fixture, result: &Comparison) -> Result<()> {
    if let Comparison::Unavailable(reason) = result
        && REQUIRED_DRAWINGS.contains(&file.sha256.as_str())
    {
        return Err(format!(
            "{}: required source formula unavailable: {reason}",
            file.name
        )
        .into());
    }
    Ok(())
}

fn compare(outcome: AnalysisOutcome, reference: &Reference) -> Result<Comparison> {
    match outcome {
        AnalysisOutcome::Available {
            atom_count,
            formula,
            ..
        } => {
            if atom_count != reference.atoms || formula != reference.formula {
                return Err(format!("Bondline {formula} ({atom_count} drawn atoms) differs from RDKit {} ({} drawn atoms)", reference.formula, reference.atoms).into());
            }
            Ok(Comparison::Matched(formula))
        }
        AnalysisOutcome::UnsupportedSource => {
            Ok(Comparison::Unavailable("unsupported_source".into()))
        }
        AnalysisOutcome::UnresolvedHydrogens { atom_id } => Ok(Comparison::Unavailable(format!(
            "unresolved_hydrogens at atom {atom_id}"
        ))),
        other => Err(format!(
            "unexpected Analysis response on nonempty pinned molecule: {other:?}"
        )
        .into()),
    }
}

impl Counts {
    fn record(
        &mut self,
        label: &str,
        reference: &Reference,
        result: Comparison,
        output: &mut impl Write,
    ) -> Result<()> {
        match result {
            Comparison::Matched(formula) => {
                self.matched += 1;
                writeln!(
                    output,
                    "{label}: Bondline {formula} matched RDKit ({} drawn atoms)",
                    reference.atoms
                )?;
            }
            Comparison::Unavailable(reason) => {
                self.unavailable += 1;
                writeln!(
                    output,
                    "{label}: {reason}; RDKit reference {}",
                    reference.formula
                )?;
            }
        }
        Ok(())
    }
}

fn open(path: &Path, bytes: &[u8], format: Format) -> Result<Engine> {
    let command = match format {
        Format::Cdxml => "document.open_cdxml",
        Format::Cdx => "document.open_cdx",
        Format::Mol | Format::MolV3000 => "document.open_mol",
        Format::Sdf => "document.open_sdf",
        Format::Smiles => "document.open_smiles",
    };
    let mut app = Engine::default();
    app.dispatch(command, json!({"path":path}))?;
    let opened = match format {
        Format::Cdxml => app.stored_document()?.export_cdxml()?,
        Format::Cdx => app.stored_document()?.export_cdx()?,
        Format::Mol | Format::MolV3000 => app.stored_document()?.export_mol()?,
        Format::Sdf => app.content().sdf()?.as_bytes(),
        Format::Smiles => app.content().smiles_file()?.as_bytes(),
    };
    if opened != bytes {
        return Err("registry opened different bytes from the SHA-256 verified corpus".into());
    }
    Ok(app)
}

impl Coverage {
    pub(super) fn check(
        &mut self,
        file: &Fixture,
        path: &Path,
        bytes: &[u8],
        output: &mut impl Write,
    ) -> Result<()> {
        let name = &file.name;
        let format = match file.format {
            Format::Cdxml => "cdxml",
            Format::Cdx => "cdx",
            Format::Mol | Format::MolV3000 => "mol",
            Format::Sdf => "sdf",
            Format::Smiles => "smiles",
        };
        let independent = super::python_output(
            include_str!("../../oracles/corpus_formula.py"),
            &[format],
            bytes,
        )?;
        let refs: Vec<Reference> = serde_json::from_slice(&independent)?;
        let mut app = open(path, bytes, file.format)?;
        let outcome = analysis(&mut app)?;
        if !matches!(file.format, Format::Sdf | Format::Smiles) {
            let [reference] = refs.as_slice() else {
                return Err("expected a whole-drawing reference".into());
            };
            let result = compare(outcome, reference)?;
            require_proven_formula(file, &result)?;
            return self
                .files
                .record(&format!("{name} drawing"), reference, result, output);
        }
        // A collection can contain unrelated structures. There is no reference formula
        // for adding its records together. Compare a molecular result only on that record.
        if !matches!(outcome, AnalysisOutcome::UnsupportedSource) {
            return Err(
                format!("{name}: unexpected collection-scope formula result {outcome:?}").into(),
            );
        }
        self.files.unavailable += 1;
        writeln!(
            output,
            "{name} collection: unsupported_source; {} molecule record(s)",
            refs.len()
        )?;
        if matches!(file.format, Format::Sdf) {
            let source = SdfFile::parse(bytes)?;
            if refs.len() != source.records().len() {
                return Err("SDF record count differs from RDKit mol-block scan".into());
            }
            let transform = MolTransform::new(24.0, Point2::new(0., 0.)?)?;
            let mut isolated_blocks = Counts::default();
            for (index, (record, reference)) in source.records().iter().zip(&refs).enumerate() {
                let drawing = StoredDocument::from_mol(
                    Arc::from(record.mol_bytes()),
                    PageSize::A4,
                    transform,
                )?;
                let result = analysis(&mut Engine::from_document(drawing))?;
                let isolated = compare(result, reference)?;
                let id = std::num::NonZeroU32::try_from(u32::try_from(index + 1)?)?;
                let mut request = app.analysis_properties_request(false);
                request.sdf_record_id = Some(id);
                let report = app
                    .execute(Command::AnalysisProperties(request))?
                    .analysis
                    .ok_or("missing open SDF record analysis")?;
                if !matches!(report.scope, AnalysisScope::SdfRecord) || report.record_id != Some(id)
                {
                    return Err(
                        format!("{name}: analysis returned a different record scope").into(),
                    );
                }
                let direct = compare(report.outcome, reference)?;
                if matches!(isolated, Comparison::Matched(_))
                    && matches!(direct, Comparison::Unavailable(_))
                {
                    return Err(
                        format!("{name}: record {id} was available only after extraction").into(),
                    );
                }
                isolated_blocks.record(
                    &format!("{name} isolated MOL block {id}"),
                    reference,
                    isolated,
                    output,
                )?;
                self.records.record(
                    &format!("{name} open SDF record {id}"),
                    reference,
                    direct,
                    output,
                )?;
            }
        } else {
            let raw = app.content().smiles_file()?;
            if refs.len() != raw.records().len() {
                return Err("SMILES record count differs from RDKit notation scan".into());
            }
            for (record, reference) in raw.records().iter().zip(&refs) {
                let id = record.id().get();
                let mut drawing = open(path, bytes, file.format)?;
                let params = json!({"expected_revision":drawing.state().revision,"selection":{"mode":"record","record_id":id},
                    "bond_length_pt":48,"origin_x_pt":72,"origin_y_pt":72,"page_width_pt":595,"page_height_pt":842});
                let result = match drawing.dispatch("document.import_smiles_records", params) {
                    Ok(_) => compare(analysis(&mut drawing)?, reference)?,
                    Err(error) if error.info().code == "unsupported_content" => {
                        Comparison::Unavailable(format!("import refused ({error})"))
                    }
                    Err(error) => return Err(error.into()),
                };
                self.records.record(
                    &format!("{name} SMILES record {id}"),
                    reference,
                    result,
                    output,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bondline_model::{Atom, AtomId, Element, Fragment, FragmentId, PageId};

    #[test]
    fn a_required_formula_cannot_regress_to_an_unavailable_source() -> Result<()> {
        let manifest: super::super::Manifest =
            serde_json::from_slice(include_bytes!("../../../docs/corpus/manifest.json"))?;
        for drawing in manifest.files {
            let reason = require_proven_formula(
                &drawing,
                &Comparison::Unavailable("unsupported_source".into()),
            )
            .err()
            .ok_or("a previously supported source must keep its chemistry coverage")?;
            assert!(
                reason
                    .to_string()
                    .contains("required source formula unavailable: unsupported_source")
            );
        }
        Ok(())
    }

    #[test]
    fn formula_oracle_checks_isotopes_carbon_free_hill_order_and_drawn_atom_count() -> Result<()> {
        for (notation, element, mass, attached, expected, wrong) in [
            ("[13CH4]", 6, Some(13), 4, "[13C]H4", "CH4"),
            ("Br", 35, None, 1, "BrH", "HBr"),
        ] {
            let mut atom = Atom::new(
                AtomId::new(1).ok_or("atom id")?,
                Element::try_from(element)?,
                None,
            );
            atom.isotope = mass.and_then(std::num::NonZeroU16::new);
            atom.explicit_hydrogens = Some(attached);
            let mut stored = StoredDocument::default();
            stored.model_mut().append_fragments(
                PageId::new(1).ok_or("page id")?,
                vec![Fragment::new(
                    FragmentId::new(1).ok_or("fragment id")?,
                    vec![atom],
                    vec![],
                )?],
            )?;
            let outcome = analysis(&mut Engine::from_document(stored))?;
            let answers: Vec<Reference> = serde_json::from_slice(&super::super::python_output(
                include_str!("../../oracles/corpus_formula.py"),
                &["smiles"],
                notation.as_bytes(),
            )?)?;
            let [mut reference] =
                <[Reference; 1]>::try_from(answers).map_err(|_| "expected one reference")?;
            assert_eq!(reference.formula, expected);
            assert!(
                matches!(compare(outcome.clone(), &reference)?, Comparison::Matched(formula) if formula == expected)
            );
            reference.formula = wrong.into();
            assert!(compare(outcome.clone(), &reference).is_err());
            reference.formula = expected.into();
            reference.atoms += 1;
            assert!(compare(outcome, &reference).is_err());
        }
        Ok(())
    }
}
