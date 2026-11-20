use bondline_commands::{
    AnalysisMass, AnalysisOutcome, AnalysisReport, AnalysisScope, CaptureStamp, Command,
    ElementalAnalysis, Engine, UiAnalysis,
};
use eframe::egui;

#[derive(Default)]
pub(super) struct AnalysisWindow {
    cached: Option<(CaptureStamp, u64, Result<AnalysisReport, String>)>,
    record_number: u32,
}

impl AnalysisWindow {
    pub(super) fn show(&mut self, ctx: &egui::Context, engine: &mut Engine) {
        if !engine.analysis_window_open() {
            self.cached = None;
            self.record_number = 1;
            return;
        }
        if let Some((stamp, _, _)) = &self.cached
            && !engine.capture_stamp_matches(stamp)
        {
            if self.record_number > 1 && !engine.same_sdf_analysis_record(stamp, self.record_number)
            {
                self.record_number = 1;
            }
            self.cached = None;
        }
        let mut open = true;
        egui::Window::new("Analysis")
            .open(&mut open)
            .default_pos(egui::pos2(
                (ctx.screen_rect().right() - 348.0).max(0.0),
                76.0,
            ))
            .default_width(298.0)
            .show(ctx, |ui| {
                let record_count = engine
                    .content()
                    .sdf()
                    .ok()
                    .and_then(|source| u32::try_from(source.records().len()).ok())
                    .unwrap_or(0);
                let mut request = engine.analysis_properties_request(true);
                if record_count > 0 {
                    if choose_record(ui, &mut self.record_number, record_count) {
                        self.cached = None;
                    }
                    request.sdf_record_id = std::num::NonZeroU32::new(self.record_number);
                }
                let epoch = request.expected_selection_epoch;
                if self
                    .cached
                    .as_ref()
                    .is_none_or(|(_, previous, _)| *previous != epoch)
                {
                    let report = engine
                        .execute(Command::AnalysisProperties(request))
                        .map_err(|error| error.to_string())
                        .and_then(|state| {
                            state
                                .analysis
                                .ok_or_else(|| "The analysis result is unavailable.".into())
                        });
                    self.cached = Some((engine.capture_stamp(), epoch, report));
                }
                let Some((_, _, report)) = &self.cached else {
                    return;
                };
                match report {
                    Ok(report) => display(ui, report, engine),
                    Err(message) => {
                        ui.label(message);
                    }
                }
            });
        if !open {
            let _ = engine.execute(Command::UiAnalysis(UiAnalysis {
                expected_revision: engine.analysis_request().expected_revision,
                visible: false,
            }));
            self.cached = None;
        }
    }
}

fn choose_record(ui: &mut egui::Ui, number: &mut u32, count: u32) -> bool {
    *number = (*number).clamp(1, count);
    let previous = *number;
    if count > 1 {
        ui.horizontal(|ui| {
            let label = ui.label("Record number");
            ui.add(egui::DragValue::new(number).range(1..=count).speed(1.0))
                .labelled_by(label.id);
        });
        *number = (*number).clamp(1, count);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(*number > 1, egui::Button::new("Previous molecule"))
                .clicked()
            {
                *number -= 1;
            }
            if ui
                .add_enabled(*number < count, egui::Button::new("Next molecule"))
                .clicked()
            {
                *number += 1;
            }
        });
    }
    ui.label(format!("SDF record {number} of {count}"));
    *number != previous
}

fn display(ui: &mut egui::Ui, report: &AnalysisReport, engine: &Engine) {
    match report.scope {
        AnalysisScope::Document => {
            ui.weak("Entire drawing");
        }
        AnalysisScope::Selection => {
            ui.weak("Current selection");
        }
        AnalysisScope::SdfRecord => {}
    };
    match &report.outcome {
        AnalysisOutcome::Available {
            atom_count,
            fragment_count,
            partial_fragments,
            defaulted_hydrogen_atoms,
            formula,
            masses,
            elemental_analysis,
        } => {
            let title = if *partial_fragments {
                "Selected-atom composition"
            } else if *fragment_count > 1 {
                "Combined formula"
            } else {
                "Molecular formula"
            };
            ui.label(title);
            egui::ScrollArea::vertical()
                .max_height(132.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(formula).size(22.0).strong())
                            .wrap()
                            .selectable(true),
                    );
                });
            if let Some(masses) = masses {
                let (weight, exact) = if *partial_fragments {
                    ("Weight contribution", "Exact mass contribution")
                } else {
                    ("Molecular weight", "Exact mass")
                };
                mass_row(ui, weight, masses.molecular_weight, 3, "");
                mass_row(ui, exact, masses.exact_mass_da, 6, " Da");
            }
            if let Some(elements) = elemental_analysis {
                elemental_table(ui, elements, *partial_fragments);
            }
            ui.weak(format!(
                "{atom_count} drawn {} · {fragment_count} {}",
                if *atom_count == 1 { "atom" } else { "atoms" },
                if *fragment_count == 1 {
                    "fragment"
                } else {
                    "fragments"
                }
            ));
            if let Some(count) = defaulted_hydrogen_atoms {
                let unit = if count.get() == 1 { "atom" } else { "atoms" };
                let source = match engine.content().source_info().map(|s| s.format) {
                    Some("mol") => "MOL",
                    Some("sdf") if matches!(report.scope, AnalysisScope::SdfRecord) => "MOL",
                    Some("cdxml") => "CDXML",
                    Some("cdx") => "CDX",
                    _ => "source",
                };
                ui.label(format!(
                    "The {source} valence default supplies hydrogen counts for {count} {unit}."
                ));
            }
            if *partial_fragments {
                ui.label("Includes the selected atoms and their attached hydrogens. Cut bonds are uncapped.");
            }
        }
        AnalysisOutcome::Empty => {
            ui.label("No atoms in this drawing.");
        }
        AnalysisOutcome::UnresolvedHydrogens { atom_id } => {
            ui.label(format!(
                "Formula and masses unavailable. Atom {atom_id} has an unspecified attached-hydrogen count."
            ));
        }
        AnalysisOutcome::UnsupportedSource => {
            ui.label(
                "Formula and masses unavailable. Complete chemical coverage is required for this source.",
            );
        }
        AnalysisOutcome::RecordByteLimit { limit } => {
            ui.label(format!(
                "This MOL block exceeds the {limit}-byte record analysis limit."
            ));
        }
        AnalysisOutcome::LimitExceeded { limit } => {
            ui.label(format!("Formula and masses unavailable. The analysis limit is {limit} drawn atoms and {limit} fragments."));
        }
    }
}

fn mass_row(ui: &mut egui::Ui, title: &str, mass: AnalysisMass, decimals: usize, unit: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(title);
        match mass {
            AnalysisMass::Value(value) => {
                ui.add(egui::Label::new(format!("{value:.decimals$}{unit}")).selectable(true));
            }
            AnalysisMass::Unavailable(reason) => {
                ui.weak(format!("Unavailable ({reason})"));
            }
        }
    });
}

fn elemental_table(ui: &mut egui::Ui, analysis: &ElementalAnalysis, partial: bool) {
    use std::fmt::Write;
    match analysis {
        ElementalAnalysis::Available { elements } => {
            ui.label(if partial {
                "Selected atoms (% by weight)"
            } else {
                "Elemental analysis (% by weight)"
            });
            // A single selectable text block copies the headings and aligned rows.
            let mut table = String::from("Element        Weight   Percent");
            for row in elements {
                let _ = write!(
                    table,
                    "\n{:<7} {:>13.3} {:>8.2}%",
                    row.symbol, row.weight_contribution, row.percent_by_weight
                );
            }
            egui::ScrollArea::vertical()
                .id_salt("elemental-analysis")
                .max_height(112.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(table).monospace()).selectable(true),
                    );
                });
        }
        ElementalAnalysis::Unavailable(reason) => {
            ui.weak(format!("Elemental analysis unavailable ({reason})"));
        }
    }
}
