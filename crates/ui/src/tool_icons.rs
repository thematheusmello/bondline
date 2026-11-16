//! Original Bondline roadmap glyphs; presentation only, never tool behavior.
use super::tool_catalogue::{Entry, Icon};
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke};

fn point(r: Rect, a: [f32; 2]) -> Pos2 {
    egui::pos2(r.left() + a[0] * r.width(), r.top() + a[1] * r.height())
}
fn pen(r: Rect, c: Color32) -> Stroke {
    Stroke::new(r.width().min(r.height()) / 12.8, c)
}
fn path(p: &Painter, r: Rect, a: &[[f32; 2]], c: Color32) {
    p.add(Shape::line(
        a.iter().map(|a| point(r, *a)).collect(),
        pen(r, c),
    ));
}
fn looped(p: &Painter, r: Rect, a: &[[f32; 2]], c: Color32, fill: bool) {
    let points = a.iter().map(|a| point(r, *a)).collect();
    p.add(if fill {
        Shape::convex_polygon(points, c, pen(r, c))
    } else {
        Shape::closed_line(points, pen(r, c))
    });
}
fn oval(p: &Painter, r: Rect, a: [f32; 4], c: Color32, fill: bool) {
    let points: Vec<_> = (0..24)
        .map(|i| {
            let t = i as f32 * std::f32::consts::TAU / 24.0;
            [a[0] + a[2] * t.cos(), a[1] + a[3] * t.sin()]
        })
        .collect();
    looped(p, r, &points, c, fill);
}
fn head(p: &Painter, r: Rect, tip: [f32; 2], tail: [f32; 2], c: Color32, half: bool) {
    let tip = point(r, tip);
    let back = (point(r, tail) - tip).normalized() * r.width().min(r.height()) * 0.18;
    let side = egui::vec2(-back.y, back.x) * 0.55;
    p.line_segment([tip + back + side, tip], pen(r, c));
    if !half {
        p.line_segment([tip, tip + back - side], pen(r, c));
    }
}
fn arrow(p: &Painter, r: Rect, a: [f32; 2], b: [f32; 2], c: Color32) {
    path(p, r, &[a, b], c);
    head(p, r, b, a, c, false);
}
fn tag(p: &Painter, r: Rect, word: &str, c: Color32) {
    p.text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        word,
        egui::FontId::proportional(r.width().min(r.height()) * 0.43),
        c,
    );
}

pub(super) fn paint(painter: &Painter, rect: Rect, entry: &Entry, color: Color32) {
    let r = rect.shrink(6.0);
    if !r.is_finite() || !r.is_positive() {
        return;
    }
    let p = painter;
    let c = color;
    let v = entry.variant;
    match &entry.icon {
        Icon::Lasso => {
            looped(
                p,
                r,
                &[
                    [0.12, 0.32],
                    [0.27, 0.12],
                    [0.62, 0.10],
                    [0.85, 0.28],
                    [0.89, 0.57],
                    [0.68, 0.78],
                    [0.34, 0.82],
                    [0.13, 0.64],
                ],
                c,
                false,
            );
            path(p, r, &[[0.13, 0.64], [0.10, 0.90], [0.30, 0.89]], c);
        }
        Icon::Marquee => {
            for (a, b) in [
                ([0.1, 0.3], [0.1, 0.1]),
                ([0.1, 0.1], [0.3, 0.1]),
                ([0.7, 0.1], [0.9, 0.1]),
                ([0.9, 0.1], [0.9, 0.3]),
                ([0.9, 0.7], [0.9, 0.9]),
                ([0.9, 0.9], [0.7, 0.9]),
                ([0.3, 0.9], [0.1, 0.9]),
                ([0.1, 0.9], [0.1, 0.7]),
            ] {
                path(p, r, &[a, b], c);
            }
        }
        Icon::Perspective => {
            looped(
                p,
                r,
                &[[0.1, 0.25], [0.65, 0.1], [0.9, 0.75], [0.35, 0.9]],
                c,
                false,
            );
            path(p, r, &[[0.1, 0.25], [0.9, 0.75]], c);
        }
        Icon::Eraser => {
            looped(
                p,
                r,
                &[[0.1, 0.6], [0.55, 0.12], [0.9, 0.44], [0.45, 0.9]],
                c,
                false,
            );
            path(p, r, &[[0.27, 0.43], [0.62, 0.75]], c);
        }
        Icon::Bond => match v {
            "wedge" => looped(p, r, &[[0.1, 0.5], [0.88, 0.2], [0.88, 0.8]], c, true),
            "hashed" | "hashed_wedge" => {
                for i in 0..6 {
                    let x = 0.15 + i as f32 * 0.14;
                    let h = if v == "hashed_wedge" {
                        0.03 + i as f32 * 0.045
                    } else {
                        0.18
                    };
                    path(p, r, &[[x, 0.5 - h], [x, 0.5 + h]], c);
                }
            }
            "dative" => arrow(p, r, [0.12, 0.5], [0.88, 0.5], c),
            "hydrogen" => {
                for i in 0..5 {
                    oval(p, r, [0.16 + i as f32 * 0.17, 0.5, 0.035, 0.035], c, true);
                }
            }
            "crossed_double" => {
                path(p, r, &[[0.1, 0.28], [0.9, 0.72]], c);
                path(p, r, &[[0.1, 0.72], [0.9, 0.28]], c);
            }
            "wavy" => {
                let a: Vec<_> = (0..25)
                    .map(|i| {
                        let x = i as f32 / 24.0;
                        [
                            0.1 + 0.8 * x,
                            0.5 + 0.1 * (x * std::f32::consts::TAU * 2.0).sin(),
                        ]
                    })
                    .collect();
                path(p, r, &a, c);
            }
            "dashed" | "dashed_double" | "aromatic_tautomeric" => {
                let ys: &[f32] = if v == "dashed" { &[0.5] } else { &[0.35, 0.65] };
                for y in ys {
                    for i in 0..4 {
                        let x = 0.12 + i as f32 * 0.2;
                        path(p, r, &[[x, *y], [x + 0.12, *y]], c);
                    }
                }
            }
            _ => {
                let ys: &[f32] = match v {
                    "double" | "bold_double" => &[0.35, 0.65],
                    "triple" => &[0.25, 0.5, 0.75],
                    "quadruple" => &[0.2, 0.4, 0.6, 0.8],
                    _ => &[0.5],
                };
                for y in ys {
                    let mut s = pen(r, c);
                    if v == "bold" || v == "bold_double" {
                        s.width *= 1.8;
                    }
                    p.line_segment([point(r, [0.1, *y]), point(r, [0.9, *y])], s);
                }
            }
        },
        Icon::Chain => path(
            p,
            r,
            if v == "snaking" {
                &[
                    [0.1, 0.8],
                    [0.3, 0.5],
                    [0.5, 0.8],
                    [0.7, 0.5],
                    [0.5, 0.2],
                    [0.8, 0.1],
                ]
            } else {
                &[
                    [0.1, 0.65],
                    [0.3, 0.35],
                    [0.5, 0.65],
                    [0.7, 0.35],
                    [0.9, 0.65],
                ]
            },
            c,
        ),
        Icon::Chair => {
            let mut a = [
                [0.1, 0.65],
                [0.3, 0.3],
                [0.65, 0.4],
                [0.9, 0.15],
                [0.72, 0.65],
                [0.4, 0.8],
            ];
            if v == "2" {
                for q in &mut a {
                    q[1] = 1.0 - q[1];
                }
            }
            looped(p, r, &a, c, false);
        }
        Icon::Ring => {
            let n = match v {
                "single_3" => 3,
                "single_4" => 4,
                "single_5" | "cyclopentadiene" => 5,
                "single_7" => 7,
                "single_8" => 8,
                _ => 6,
            };
            let a: Vec<_> = (0..n)
                .map(|i| {
                    let t = i as f32 * std::f32::consts::TAU / n as f32;
                    [0.5 + 0.38 * t.cos(), 0.5 + 0.38 * t.sin()]
                })
                .collect();
            looped(p, r, &a, c, false);
        }
        Icon::Arrow => match v {
            "block" => looped(
                p,
                r,
                &[
                    [0.1, 0.35],
                    [0.58, 0.35],
                    [0.58, 0.15],
                    [0.9, 0.5],
                    [0.58, 0.85],
                    [0.58, 0.65],
                    [0.1, 0.65],
                ],
                c,
                false,
            ),
            "equilibrium" => {
                arrow(p, r, [0.12, 0.35], [0.88, 0.35], c);
                arrow(p, r, [0.88, 0.65], [0.12, 0.65], c);
            }
            "curved" => {
                path(
                    p,
                    r,
                    &[
                        [0.12, 0.8],
                        [0.18, 0.4],
                        [0.4, 0.2],
                        [0.7, 0.23],
                        [0.88, 0.45],
                    ],
                    c,
                );
                head(p, r, [0.88, 0.45], [0.7, 0.23], c, false);
            }
            "retrosynthesis" => {
                path(p, r, &[[0.12, 0.4], [0.72, 0.4]], c);
                path(p, r, &[[0.12, 0.6], [0.72, 0.6]], c);
                path(p, r, &[[0.65, 0.2], [0.9, 0.5], [0.65, 0.8]], c);
            }
            _ => arrow(p, r, [0.12, 0.5], [0.88, 0.5], c),
        },
        Icon::Electron => {
            path(
                p,
                r,
                &[[0.12, 0.8], [0.25, 0.45], [0.55, 0.25], [0.85, 0.3]],
                c,
            );
            head(p, r, [0.85, 0.3], [0.55, 0.25], c, v == "single");
        }
        Icon::Fragmentation => {
            path(p, r, &[[0.1, 0.7], [0.3, 0.3], [0.43, 0.5]], c);
            path(p, r, &[[0.6, 0.5], [0.75, 0.7], [0.9, 0.3]], c);
            path(
                p,
                r,
                &[[0.5, 0.15], [0.44, 0.38], [0.58, 0.33], [0.5, 0.85]],
                c,
            );
        }
        Icon::Mapping => {
            oval(p, r, [0.2, 0.5, 0.12, 0.12], c, false);
            oval(p, r, [0.8, 0.5, 0.12, 0.12], c, false);
            arrow(p, r, [0.38, 0.5], [0.64, 0.5], c);
        }
        Icon::Shape => match v {
            "circle" => oval(p, r, [0.5, 0.5, 0.38, 0.38], c, false),
            "oval" => oval(p, r, [0.5, 0.5, 0.4, 0.25], c, false),
            "line" => path(p, r, &[[0.1, 0.85], [0.9, 0.15]], c),
            _ => looped(
                p,
                r,
                &[[0.1, 0.15], [0.9, 0.15], [0.9, 0.85], [0.1, 0.85]],
                c,
                false,
            ),
        },
        Icon::Arc => {
            let angle = if v.ends_with("120") {
                120.0_f32
            } else if v.ends_with("180") {
                180.0
            } else if v.ends_with("270") {
                270.0
            } else {
                90.0
            };
            let a: Vec<_> = (0..25)
                .map(|i| {
                    let t = -std::f32::consts::FRAC_PI_2 + angle.to_radians() * i as f32 / 24.0;
                    [0.5 + 0.38 * t.cos(), 0.5 + 0.38 * t.sin()]
                })
                .collect();
            for (i, pair) in a.windows(2).enumerate() {
                if !v.starts_with("dashed") || i % 4 < 2 {
                    path(p, r, pair, c);
                }
            }
        }
        Icon::Bracket => {
            let a: &[[f32; 2]] = if v.ends_with("brace") {
                &[
                    [0.35, 0.1],
                    [0.22, 0.1],
                    [0.22, 0.35],
                    [0.1, 0.5],
                    [0.22, 0.65],
                    [0.22, 0.9],
                    [0.35, 0.9],
                ]
            } else if v.ends_with("parenthesis") {
                &[
                    [0.35, 0.1],
                    [0.2, 0.28],
                    [0.15, 0.5],
                    [0.2, 0.72],
                    [0.35, 0.9],
                ]
            } else {
                &[[0.35, 0.1], [0.15, 0.1], [0.15, 0.9], [0.35, 0.9]]
            };
            path(p, r, a, c);
            if v.starts_with("paired") {
                let b: Vec<_> = a.iter().map(|a| [1.0 - a[0], a[1]]).collect();
                path(p, r, &b, c);
            }
        }
        Icon::Dagger => {
            path(p, r, &[[0.5, 0.1], [0.5, 0.9]], c);
            path(p, r, &[[0.25, 0.35], [0.75, 0.35]], c);
        }
        Icon::Curve => {
            let a = [
                [0.1, 0.8],
                [0.2, 0.4],
                [0.4, 0.2],
                [0.65, 0.3],
                [0.75, 0.6],
                [0.9, 0.7],
            ];
            path(p, r, &a, c);
            if v == "edit" {
                for q in [a[0], a[2], a[5]] {
                    oval(p, r, [q[0], q[1], 0.055, 0.055], c, false);
                }
            }
        }
        Icon::Text => {
            path(p, r, &[[0.15, 0.15], [0.85, 0.15]], c);
            path(p, r, &[[0.5, 0.15], [0.5, 0.85]], c);
            path(p, r, &[[0.3, 0.85], [0.7, 0.85]], c);
        }
        Icon::Table | Icon::PeriodicTable => {
            looped(
                p,
                r,
                &[[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]],
                c,
                false,
            );
            for x in [0.36, 0.64] {
                path(p, r, &[[x, 0.1], [x, 0.9]], c);
            }
            for y in [0.36, 0.64] {
                path(p, r, &[[0.1, y], [0.9, y]], c);
            }
            if matches!(&entry.icon, Icon::PeriodicTable) {
                oval(p, r, [0.23, 0.23, 0.045, 0.045], c, true);
            }
        }
        Icon::Template => {
            looped(
                p,
                r,
                &[[0.1, 0.1], [0.7, 0.1], [0.7, 0.7], [0.1, 0.7]],
                c,
                false,
            );
            looped(
                p,
                r,
                &[[0.3, 0.3], [0.9, 0.3], [0.9, 0.9], [0.3, 0.9]],
                c,
                false,
            );
        }
        Icon::CharacterMap => tag(p, r, "α", c),
        Icon::Label => {
            tag(p, r, "R", c);
            arrow(p, r, [0.12, 0.85], [0.88, 0.85], c);
        }
        Icon::Name => {
            tag(p, r, if v == "to_name" { "N" } else { "C" }, c);
            arrow(
                p,
                r,
                if v == "to_name" {
                    [0.85, 0.85]
                } else {
                    [0.15, 0.85]
                },
                if v == "to_name" {
                    [0.15, 0.85]
                } else {
                    [0.85, 0.85]
                },
                c,
            );
        }
        Icon::Chromatography => {
            if v == "spot" {
                oval(p, r, [0.5, 0.5, 0.25, 0.32], c, false);
            } else {
                looped(
                    p,
                    r,
                    &[[0.12, 0.1], [0.88, 0.1], [0.88, 0.9], [0.12, 0.9]],
                    c,
                    false,
                );
                path(p, r, &[[0.12, 0.78], [0.88, 0.78]], c);
                for (x, y) in [(0.3, 0.35), (0.65, 0.6)] {
                    if v == "gel" {
                        path(p, r, &[[x - 0.1, y], [x + 0.1, y]], c);
                    } else {
                        oval(p, r, [x, y, 0.055, 0.075], c, true);
                    }
                }
            }
        }
        Icon::Symbol => symbol(p, r, v, c),
        Icon::Orbital => match v {
            "15" => oval(p, r, [0.5, 0.5, 0.32, 0.32], c, false),
            "17" => looped(
                p,
                r,
                &[
                    [0.15, 0.5],
                    [0.5, 0.18],
                    [0.8, 0.3],
                    [0.88, 0.5],
                    [0.8, 0.7],
                    [0.5, 0.82],
                ],
                c,
                false,
            ),
            "20" => {
                for (x, y) in [(0.3, 0.3), (0.7, 0.3), (0.3, 0.7), (0.7, 0.7)] {
                    oval(p, r, [x, y, 0.15, 0.15], c, false);
                }
            }
            "16" | "19" => {
                oval(p, r, [0.3, 0.5, 0.18, 0.25], c, false);
                oval(p, r, [0.7, 0.5, 0.18, 0.25], c, false);
            }
            _ => {
                oval(p, r, [0.5, 0.3, 0.18, 0.18], c, false);
                oval(p, r, [0.5, 0.7, 0.18, 0.18], c, false);
                if v == "21" {
                    oval(p, r, [0.5, 0.5, 0.3, 0.09], c, false);
                }
            }
        },
        Icon::Query => {
            looped(
                p,
                r,
                &[[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]],
                c,
                false,
            );
            if v == "24" {
                path(p, r, &[[0.32, 0.3], [0.32, 0.68], [0.75, 0.68]], c);
                path(p, r, &[[0.32, 0.53], [0.47, 0.53], [0.47, 0.68]], c);
            } else {
                tag(
                    p,
                    r,
                    match v {
                        "22" => "*",
                        "23" => "R",
                        "25" => "d",
                        "26" => "φ",
                        "27" => "L",
                        "28" => "P",
                        "29" => "r",
                        "30" => "n",
                        "31" => "•",
                        "32" => "rac",
                        "33" => "abs",
                        "34" => "rel",
                        _ => "R",
                    },
                    c,
                );
            }
        }
        Icon::Biology => {
            if v == "04" || v == "03" {
                for sign in [-1.0, 1.0] {
                    let a: Vec<_> = (0..25)
                        .map(|i| {
                            let t = i as f32 / 24.0;
                            [
                                0.5 + sign * 0.28 * (t * std::f32::consts::TAU * 1.5).sin(),
                                0.12 + 0.76 * t,
                            ]
                        })
                        .collect();
                    path(p, r, &a, c);
                }
                if v == "04" {
                    for i in 0..5 {
                        let y = 0.2 + i as f32 * 0.15;
                        path(p, r, &[[0.3, y], [0.7, y]], c);
                    }
                }
            } else {
                oval(p, r, [0.5, 0.5, 0.39, 0.32], c, false);
                tag(
                    p,
                    r,
                    match v {
                        "01" => "R",
                        "02" | "13" | "14" => "G",
                        "09" => "P",
                        "10" => "r",
                        "11" => "tR",
                        "12" => "I",
                        "15" => "Ig",
                        "16" => "G",
                        "17" => "ER",
                        "18" => "M",
                        "19" => "C",
                        _ => "B",
                    },
                    c,
                );
            }
        }
        Icon::Helm => {
            path(p, r, &[[0.18, 0.74], [0.5, 0.74], [0.82, 0.74]], c);
            for x in [0.18, 0.5, 0.82] {
                oval(p, r, [x, 0.74, 0.07, 0.07], c, false);
            }
            if v == "25" {
                arrow(p, r, [0.2, 0.22], [0.8, 0.22], c);
                arrow(p, r, [0.8, 0.45], [0.2, 0.45], c);
            } else {
                let label = match v {
                    "21" => "AA",
                    "22" => "RNA",
                    "23" => "C",
                    "24" => "B",
                    "26" => "DB",
                    "27" => "M",
                    "28" => "E",
                    "29" => "S",
                    "30" => "F",
                    "31" => "seq",
                    "32" => "lib",
                    _ => "M",
                };
                tag(
                    p,
                    r.translate(egui::vec2(0.0, -r.height() * 0.17)),
                    label,
                    c,
                );
            }
        }
    }
}

fn symbol(p: &Painter, r: Rect, v: &str, c: Color32) {
    match v {
        "01" | "02" | "03" | "04" | "05" | "06" => {
            if v == "01" || v == "03" {
                oval(p, r, [0.5, 0.5, 0.38, 0.38], c, false);
            }
            path(p, r, &[[0.3, 0.5], [0.7, 0.5]], c);
            if v == "01" || v == "02" || v == "05" {
                path(p, r, &[[0.5, 0.3], [0.5, 0.7]], c);
            }
            if v == "05" || v == "06" {
                oval(p, r, [0.82, 0.25, 0.04, 0.04], c, true);
            }
        }
        "07" => oval(p, r, [0.5, 0.5, 0.07, 0.07], c, true),
        "08" => {
            for x in [0.36, 0.64] {
                oval(p, r, [x, 0.5, 0.055, 0.055], c, true);
            }
        }
        "09" => path(p, r, &[[0.28, 0.5], [0.72, 0.5]], c),
        "10" | "11" => {
            tag(p, r, "H", c);
            if v == "10" {
                oval(p, r, [0.84, 0.25, 0.04, 0.04], c, true);
            } else {
                path(p, r, &[[0.72, 0.25], [0.9, 0.25]], c);
            }
        }
        "12" => {
            path(p, r, &[[0.1, 0.5], [0.3, 0.5]], c);
            looped(
                p,
                r,
                &[[0.3, 0.5], [0.55, 0.22], [0.82, 0.5], [0.55, 0.78]],
                c,
                false,
            );
        }
        "13" => {
            path(p, r, &[[0.1, 0.5], [0.3, 0.5]], c);
            oval(p, r, [0.6, 0.5, 0.27, 0.27], c, true);
        }
        _ => {
            path(p, r, &[[0.1, 0.5], [0.42, 0.5]], c);
            oval(p, r, [0.65, 0.5, 0.22, 0.22], c, false);
        }
    }
}
