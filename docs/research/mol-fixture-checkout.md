# Pinned MOL checkout bytes

The first codec checkpoint9e3777e passed local, hosted Mac/Linux and all three sanitizer
targets, but its Windows test job132157023978 failed three MOL assertions:
https://github.com/thematheusmello/bondline/actions/runs/43504072080 .

Git's Windows checkout converted the committed LF fixtures to CRLF. The170-byte CH3
became177bytes, breaking its independently pinned identity. The ethane test constructs a
CRLF variant from the original LF vector; converting its already rewritten checkout again
created CRCRLF. The parser correctly rejected that malformed variant. An LF-specific
terminator-removal assertion also no longer exercised its intended missing-terminator case.

The narrow correction is `/crates/io/tests/fixtures/mol/*.mol -text` in.gitattributes.
These are source-byte vectors, not platform-native text. Git's documented unset `text`
attribute disables line-ending conversion: https://git-scm.com/docs/gitattributes . No
parser, fixture byte, oracle, assertion or accepted encoding changed.

A disposable Git repository committed the exact9e3777e fixtures, then a fresh clone used
`core.autocrlf=true` with isolated system/global configuration. Without the attribute,
CH3/ethane became177/290bytes and the derived input had eight CRCRLF sequences. The exact
identity and valid-CRLF assertions failed. With the attribute, both original identities
survived; the290-byte derived valid CRLF input parsed. Deliberate CRCRLF stayed invalid.
Local logs: `target/mol-checkout-{red,green}.log`. These are checkout/configuration checks,
not new application/native-platform evidence.

| Fixture | Required bytes | Required SHA-256 |
|---|---|---|
| rdkit-CH3.mol |170|afd2352697ca20c0d580c4981fe8bd24da36c4e1661e9d2aee501a40364fd42d|
| ethane-v2000.mol |282|828c1b58b433ed3778bdb0ce7e7d02315f45850507c976a55c82fcebeebf57fc|

Reproduction needs only two throwaway repositories: commit these fixtures without the
attribute in the first, clone with `git -c core.autocrlf=true clone`, inspect byte hashes;
then commit the attribute and repeat into a fresh clone. Never rewrite the working test
fixtures or normalize bytes inside the parser/oracle to hide a checkout mutation.
Corrected hosted evidence belongs in docs/ci.md after execution.
