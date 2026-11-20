use bondline_commands::{
    AnalysisMass, AnalysisOutcome, AnalysisReport, AnalysisScope, CaptureStamp, Command, Engine,
    UiAnalysis,
};
use eframe::egui;

#[derive(Default)]
pub(super) struct AnalysisWindow {
    cached: Option<(CaptureStamp, u64, Result<AnalysisReport, String>)>,
}

impl AnalysisWindow {
    pub(super) fn show(&mut self, ctx: &egui::Context, engine: &mut Engine) {
        if !engine.analysis_window_open() {
            self.cached = None;
            return;
        }
        let epoch = engine.selection().epoch;
        if self.cached.as_ref().is_none_or(|(stamp, previous, _)| {
            !engine.capture_stamp_matches(stamp) || *previous != epoch
        }) {
            let report = engine
                .execute(Command::AnalysisProperties(engine.analysis_request()))
                .map_err(|error| error.to_string())
                .and_then(|state| {
                    state
                        .analysis
                        .ok_or_else(|| "The analysis result is unavailable.".into())
                });
            self.cached = Some((engine.capture_stamp(), epoch, report));
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
                let Some((_, _, report)) = &self.cached else {
                    return;
                };
                match report {
                    Ok(report) => display(ui, report),
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

fn display(ui: &mut egui::Ui, report: &AnalysisReport) {
    match report.scope {
        AnalysisScope::Document => ui.weak("Entire drawing"),
        AnalysisScope::Selection => ui.weak("Current selection"),
    };
    match &report.outcome {
        AnalysisOutcome::Available {
            atom_count,
            fragment_count,
            partial_fragments,
            defaulted_hydrogen_atoms,
            formula,
            masses,
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
                ui.label(format!(
                    "The CDX valence default supplies hydrogen counts for {count} {unit}."
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
