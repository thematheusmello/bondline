# Installed main Tools palette

Reviewed 2026-11-14 against `/Applications/ChemDraw.app`, bundle/build 26.1.0.6801,
on macOS 27.2 arm64. This separately scoped pass preserves the frozen
[first-pass observations](installed-ui-observations.md). The
[36-cell catalog](installed-tool-catalog.json) records exact resource identifiers and
candidate bindings. It does not mark any Bondline feature or reference workflow verified.

## Observed surface and interaction limit

The target-only native window capture shows eighteen rows and two columns. The Tools
utility window is PID 50732/window 56125, layer 3, at CoreGraphics frame (20,80,63,583).
The ignored `target/reference-tools/04-main-tools-os.png` is 109×629 PNG pixels, including
shadow, SHA-256 `9c0c6e5fe65342b848ba751bbefb34729291c85a2f6508b9b4e8307ed230005d`.
Window coordinates, screenshot pixels and declared XML dimensions are distinct; this pass
does not establish per-button native hit rectangles or a scale/padding conversion.
The metadata's on-screen attribute was absent; window-only pixels do not independently
establish foreground visibility at that instant.

Actual CUA native App documentation exposes click, drag, scroll, key/value entry and
exposed AX actions, but no hover/mouse-move method. REA's native scenario schema supports
selected AX click, scroll, value entry and wait. Neither was replaced with a synthetic
click/drag to imitate hover. No tool cell or flyout was invoked. Runtime tooltip text,
accessible control roles/actions, enabled state and active child tools remain unknown.

The known REA native-window layer-zero guard cannot select this layer-three palette;
the previously rejected selector was not repeated. A separate REA observation of the
owned blank document succeeded, evidence
`ev_bcd5889aa796349d807e40821a4fdddf74109c93ec5386e63cecb99440b0f4d1`.
Its empty AX canvas has no tool descendants. That evidence identifies the blank context,
not the palette's roles. The palette image is explicitly an ordinary macOS window-only
capture, not REA Evidence.

The public [G25 guide](https://chemistry.beloit.edu/classes/programs/ChemDraw_25_manual.pdf),
printed pages 8 and 12, supplies the older tooltip/status-bar role and small-triangle
secondary-toolbar convention. Its existing local PDF was SHA-256 verified as
`3f1ebc429df07efdd47b6218a148dc346a215043f60bf89ee9a1cb90629bd5b3`; printed page 12 was
rendered and visually reviewed. A new web fetch rejected the 17,721,447-byte file as too
large. The pinned local source remains available; this older figure is not installed26.1
runtime or platform evidence.

## Bounded static resource evidence

REA's Apple bundle-resource route inspected Info.plist, the small Main.strings dictionary,
and capped Interface Builder inventories of four and nine documents. These are
resource-only observations with null executable identity; do not replace that null with
the separate native-UI executable hash. The IB inventories preserve incomplete hierarchy,
symlink/discovery and document-budget limitations. The exact small toolbar NIB probe was
rejected as a compiled NIBArchive by the Foundation keyed-archive reader; it is not proof
of missing toolbar metadata. No deep executable provider, disassembly or target execution
was needed.

Bounded filename discovery found ordinary GUI XML. The cached REA CLI has no generic
reader for this custom XML schema, so the explicitly authorized standard-library XML
fallback was used. Only relevant layout/control metadata was extracted; image/font
payloads, long help prose and implementation data were excluded from this catalog.

| Selected resource, relative to the installed app | Bytes | SHA-256 |
|---|---:|---|
| `Contents/Resources/SpecialPurpose/GUI/ChemDraw Professional/Themes/ChemDraw/XML/Toolbars/Main_Toolbar.xml` | 9197 | `9c4871459435794747002fde6350c66b00292cf07088ebb2632f980fa527e77a` |
| `Contents/Resources/SpecialPurpose/GUI/Common/XML/Commands/CommandDefinitions.xml` | 307232 | `f7925116678d9ddc8b16d0b55194a3d736f71b6f8fea21728ff2a7f4b2d580b7` |

The main resource declares `MainToolBarID`, title Tools, a canonical eighteen-column,
two-row grid, row height and column width 26, and default vertical orientation. Visible
row = declared column and visible column = declared row is a matching inference supported
by the observed two-column arrangement. It is not a recovered runtime layout algorithm.
The resource's declared button styles are 23 toggles and 13 secondary-toolbar entries.
The small lower-right marks in the capture agree with those 13 declarations. Their menus
were not opened, and commands ending `Current` do not establish the selected child or its
persistence behavior.

Professional, Prime, Viewer and other profile directories are bundled together. The
selected Professional layout and its correspondence with the image do not establish the
active licensed edition or entitlement. Actual tier remains unknown.

## Cell map

Rows below follow the native vertical image, top to bottom. The catalog assigns IT-001
through IT-036 in row-major order and retains each exact button ID, command ID, handler
reference, short tooltip key, flyout ID and canonical resource coordinate. Names here are
concise functional paraphrases. An asterisk marks a declared secondary toolbar with a
visible small triangle; it does not mean the flyout or function was executed.

| Visible row | Left cell | Right cell |
|---:|---|---|
| 1 | IT-001 Lasso selection | IT-002 Marquee selection |
| 2 | IT-003 Structure perspective | IT-004 Fragmentation tools * |
| 3 | IT-005 Solid bond | IT-006 Eraser |
| 4 | IT-007 Multiple bond tools * | IT-008 Text |
| 5 | IT-009 Dotted/dashed bonds * | IT-010 Pen/curve tools * |
| 6 | IT-011 Hashed bond | IT-012 Arrow tools * |
| 7 | IT-013 Hashed wedge bond | IT-014 Orbital tools * |
| 8 | IT-015 Bold bond | IT-016 Drawing shapes * |
| 9 | IT-017 Wedge bond | IT-018 Bracket tools * |
| 10 | IT-019 Hollow wedge bond | IT-020 Chemical symbols * |
| 11 | IT-021 Wavy bond | IT-022 Query tools * |
| 12 | IT-023 Table | IT-024 Chromatography tools * |
| 13 | IT-025 Chain tools * | IT-026 Template tools * |
| 14 | IT-027 Three-member ring | IT-028 Four-member ring |
| 15 | IT-029 Five-member ring | IT-030 Six-member ring |
| 16 | IT-031 Seven-member ring | IT-032 Eight-member ring |
| 17 | IT-033 Chair orientation 1 | IT-034 Chair orientation 2 |
| 18 | IT-035 Cyclopentadiene ring | IT-036 Benzene ring |

For example, the letter-and-arrow glyph in row 11/right has the declared Query button
and `chooseQueryTool_Current` binding. The resource resolves that candidate role without
assigning semantics from the glyph alone. A declared handler string is not evidence that
its implementation ran. Static shortcut hints likewise remain untested on this build.

## Evidence, cleanup and next work

All complete REA JSON, bounded metadata extracts, source hashes and native screenshots
remain ignored in `target/reference-tools/`. The resource-only REA IDs are
`ev_709541a163a68c02b7d387a3be0dc8035571910c636d5d66808c25181c09e924`
(bundle plist),
`ev_6589d4cba9a000c7770901fc7b14547cab6f627a92bd40ab50d86c25be806a93`
(four-document IB inventory),
`ev_9fc2f368d5ee89e0d10254c8eeef99a0ef0c04816be6f6c57689a6c8903a767c`
(Main.strings), and
`ev_3dee88c21b6f95d15181b908419864ae2e3dd3c9305d94dd98204e7bf4932adc`
(nine-document IB inventory). The useful 36-cell custom-XML bindings are separately labeled
filesystem/XML fallback, not assigned invented REA IDs.

This native pass created only a clean owned blank window 56632. Target-only snapshots
before New, after New, before Close and after Close verify original windows 56129, 56130
and 56535 retain their identities and exact starting frames. Only the guarded owned blank
was closed, without edits, Save or a dirty prompt. The original installed hotkeys sheet
remains current and Locked; the app remains running. No preference/visibility changes,
cloud/account actions or tool-cell clicks occurred. CUA completion was issued once for
this separate pass; GUI ownership was released and all jobs are terminal.

The earlier first-pass AX partial-capture and window-restoration qualifications remain
unchanged. This catalog does not establish full accessibility, hover/flyout execution,
chemistry, pixel parity or completed M0. Next work should verify one tool's actual workflow
and independent expectations before wiring its Bondline command and original procedural
icon. Never promote these proprietary reference pictures or resource files into shipped
assets, fixtures or approved Bondline goldens.
