# CDX canonical file-end framing plan

**Recommendation:** a narrow **default canonical-envelope fallback** is source-backed and useful for existing CDX Open. Keep all successful legacy interpretations first. Do not add a producer signature, property allowlist, normalization or user-facing option. This is a proposed contract, not implementation/test evidence.

## Primary facts and their limits

- [IntroCDX](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm) specifies28 header bytes:8-byte magic,4-byte marker and16 reserved zero bytes. It describes a Document-led object tree, a zero-word terminator for every object, and separately a zero-word file end. This supports an explicit8000 root after byte28, its object end, then one file end. The zero-reserved check is a published canonical-header rule; it does not identify the writer.
- [IntroExampleSimple](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroExampleSimple.htm) instead shows Page immediately after its28-byte header and labels its last word Document end, without a further file-end word. The two listings also disagree on some bond references. Treat this as historical implicit/EOF-only compatibility evidence, not a single fully consistent chemistry fixture or proof that a separate file end is always mandatory.
- [Document](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Document.htm) identifies the top-level object as8000. Its mirrored page contains archived2010 material; these SDK pages are not evidence of a particular26.1 binary. Page/chemical validity remains outside the standalone syntax reader's promise.
- Public BSD Glysade pin `5dd743230d75ecacacdf8dcd0b0e022f13173b9c`: [CDXIO.cpp696–706](https://github.com/Glysade/chemdraw/blob/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/chemdraw/CDXIO.cpp#L696) writes22 bytes and explains the encoded Document's6 bytes occupying the historical header tail. [CDXDocument.cpp1682–1700](https://github.com/Glysade/chemdraw/blob/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/chemdraw/CDXDocument.cpp#L1682) writes that header, document and separate end. This is the **already supported** HeaderDocument shape, not Indigo's28-byte implicit shape.
- Released Apache-2.0 Indigo pin `cc2ea85187433c253c0233a13f1859a6a5298d97`: [molecule_cdxml_saver.cpp2015–2069](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/core/indigo-core/molecule/src/molecule_cdxml_saver.cpp#L2015) omits a CDXML root tag, writes its final object end at2055 and another end at2069. The original generated ethane independently establishes28-zero-header + **implicit** Document end + separate final word. This is external-writer compatibility evidence beyond the explicit shape of IntroCDX's prose.

Exact reviewed bytes are pinned in sources.json: SDK IntroCDX9132B/SHA05720e49…, simple example11483B/2131f27d…, Document32254B/72c6eb4f…; Glysade25597B/71a2942f… and53887B/dceab684…; Indigo75490B/944da51a…. Full hashes, URLs, captured sources and retrieval metadata are retained. No source is transplanted into Bondline. No new producer generation, dependency, native observation or parser execution occurred here.

## Untouched positioned ethane

Original258-byte SHA-256 `856482269dbd85640b87c11f608104dba72ebd308a686f1333a893965ed560ba`; original graph/generator/package/license and prior RDKit/TrailingData results remain frozen in the earlier investigation. The current review independently checked raw tags, IDs, payload ranges and zero words against those unchanged bytes.

| Span (half-open) | Observed structure |
| --- | --- |
| 0..28 | historical header;12..28 all zero; no8000 at22 |
| 28..142 | Document properties, beginning0805; no encoded Document opening |
| 142..148 | Page8001/id0, index5 |
| 180..186 | Fragment8003/id4, index9 |
| 186..192 /206..212 | Node8004/id5 /id6; literal position payloads retained |
| 226..232 | Bond8005/id7; references5/6 |
| 248..250 /250..252 | Bond end /Fragment end |
| 252..254 | Page end |
| 254..256 | implicit Document end, no opening record |
| 256..258 | additional final zero word; assign FileEnd in the new profile |

There are24 indexed records including both final words. A synthetic encoded Document must not be inserted or steal the first property. Header remains28 bytes, final FileEnd has parentNone/range256..258, and payload access on it refuses. Raw coordinates and chemistry are not reframed or regenerated.

## Recommended compatibility contract

1. Preserve current `Document::parse` legacy28-byte attempt and **every success**, including its reserved opaque bytes, EOF-only implicit/explicit variants and old Writer output. Preserve the existing22-byte HeaderDocument fallback and its known marker rule.
2. Only the old attempt's **TrailingData** may enable a new28-byte fallback, and only with the supported12-byte prefix plus **all16 bytes12..28 zero**. No retry after malformed/truncated fields, limit/allocation errors or unsupported magic. This guard is disjoint from HeaderDocument's8000-at22, so no input needs three parser attempts.
3. Parse a complete implicit Document or encoded8000 Document after byte28. Require its object end, **one** following zero-word FileEnd and exact EOF. Do not trim, skip, search for a plausible object or accept more ends. Preserve unknown records/length spellings and any syntactically decoded object ID, without producer/property heuristics.
4. Add descriptive `Framing::ImplicitDocumentFileEnd` and `Framing::ExplicitDocumentFileEnd`. `header()` remains28 bytes; root properties/parents/indices remain genuine; FileEnd is separate and counted toward MAX_RECORDS. The implicit root adds no encoded depth; the explicit root does. Existing64MiB/100000-record/256-encoded-depth bounds and checked property-splice suffix ownership remain unchanged.
5. **No Writer change.** Existing Writer output/fixtures/oracle identities remain exact. The new syntax profiles confer no new chemical/model/editing/export authority.

This legitimately evolves acceptance: canonical EOF-only bytes followed by one end word may now represent the new canonical file-end envelope. Existing UNKNOWN+0000 and its explicit-root wrapper have reserved bytes000102…0f, so their default TrailingData assertions remain unchanged. A **new** canonical-header variant of that unknown-record body plus FileEnd should succeed, demonstrating that no Indigo property or unknown-tag exclusion is the discriminator.

Ambiguity remains intentional: removing the final word of a new file can leave a complete legacy Document, and legacy-first parsing must still accept it as legacy. Bytes cannot establish whether that prefix is a truncated new file or an intended old file. Therefore do not promise that default parsing proves a required FileEnd, rejects every extra00 relative to every historical interpretation, or rejects all truncations of every new file. An explicit strict parsing profile would be appropriate only for a future caller that must require a file end; it is unnecessary for this bounded default Open increment.

## Connected implementation/test checks

- `crates/cdx/src/lib.rs:190` currently uses `framing != ImplicitDocument` for encoded-root closure. The new implicit variant must also remain implicit. Match encoded-root variants explicitly or use one small shared classification; do not end an implicit document when its first top-level child closes.
- `fuzz/fuzz_targets/cdx_parse.rs:7` uses the same inequality to remove an encoded root during new-tree reconstruction. Without adapting it, Indigo's first0805 Document property would be silently omitted from the comparison. Preserve every actual body property; continue excluding only genuine root markers/ends and separate FileEnd.
- Keep every old syntax/fixture/negative unchanged. Add independent full-byte/table positive checks for actual258-byte implicit output and an original explicit28-byte canonical source+FileEnd; retain opaque length edits, no-op exact source and both suffix words after splice/reparse.
- Add canonical unknown-body positive plus original noncanonical negatives, extra word/byte/nonzero end/garbage negatives, and literal record/depth/input boundary checks counting FileEnd. The complete legacy prefix at lengthN−2 remains an explicitly qualified accepted truncation. Test each selected framing's complete EOF, not an obsolete blanket rule that every appended zero is invalid.
- `io::cdx::ElementalGraph::parse` currently calls default Document::parse. This review changes no IO/application policy: source retention, unsupported projection and fully-mapped=false guards stay intact; successful syntax alone is not general CDX application compatibility.

The sole recommendation is a sourced bounded envelope with explicit compatibility/ambiguity documentation. Root owns implementation, registry/source documentation, independent tests and integration. This review starts no live jobs and modifies no Root/product/test file.

## Exact public source identities

Reviewed2026-11-15; raw public downloads remain ignored development evidence.

| Source | Bytes | SHA256 |
|---|---:|---|
| [IntroCDX.htm](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroCDX.htm) | 9132 | 05720e49f4caca8d0074e45f68a0f34d9876bb7a20951298e9943a8a49485763 |
| [IntroExampleSimple.htm](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/IntroExampleSimple.htm) | 11483 | 2131f27d6075fccbf21b262f50a0457cc31614098297cb86a7b5b67319cd9ec9 |
| [Document.htm](https://iupac.github.io/IUPAC-FAIRSpec/cdx_sdk/Document.htm) | 32254 | 72c6eb4f06c3d8ddf8950baad83bb0e9a5f62f763fa7624578646972d730dd15 |
| [CDXIO.cpp](https://raw.githubusercontent.com/Glysade/chemdraw/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/chemdraw/CDXIO.cpp) | 25597 | 71a2942f44dd6b875aee72d97c1101b874dc743d5f8f121f61f9877f3000a30b |
| [CDXDocument.cpp](https://raw.githubusercontent.com/Glysade/chemdraw/5dd743230d75ecacacdf8dcd0b0e022f13173b9c/chemdraw/CDXDocument.cpp) | 53887 | dceab68450961a57f6099ba3f981dc4ce18390211ea23930754f550a2f1ec004 |
| [molecule_cdxml_saver.cpp](https://github.com/epam/Indigo/blob/cc2ea85187433c253c0233a13f1859a6a5298d97/core/indigo-core/molecule/src/molecule_cdxml_saver.cpp) | 75490 | 944da51acdc9bc9413968ab68127eb8e68d26cfc2ac910b937cc968e4826cbf3 |

## Test-first receiving order

Independent first actual258-byte default-parser regression48668 fails at runtime with
TrailingData before production. The1667-byte declaration11cf281b25b77a07c79ea9922ab2645d99881fbd3be122e9459b429e811237ce
is frozen separately. Expanded syntax/oracle and application/wire expectations must be
frozen and their RED observed before Root changes the parser. Root owns Git, integration
and final gates; independent workers own the declaration/reader and connected review.
The fuzz reconstruction property count must be conserved, not compared only after the
same body filter has already dropped a record. Preserve first0805 as an independent
canary and classify encoded roots semantically. No UI option or Writer change is planned.
