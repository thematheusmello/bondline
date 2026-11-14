use super::{
    ComponentInput, Counts, Error, MAX_BYTES, MAX_COMPONENTS, MAX_LINE_BYTES, MAX_LINES, Role,
    parse::{Cursor, Totals, ascii, syntax, validate_mol},
};

fn add_bytes(total: &mut usize, count: usize) -> Result<(), Error> {
    *total = total
        .checked_add(count)
        .filter(|value| *value <= MAX_BYTES)
        .ok_or(Error::Resource("output exceeds 64 MiB"))?;
    Ok(())
}
fn add_lines(total: &mut usize, count: usize) -> Result<(), Error> {
    *total = total
        .checked_add(count)
        .filter(|value| *value <= MAX_LINES)
        .ok_or(Error::Resource("output exceeds 1000000 lines"))?;
    Ok(())
}

/// Create a legacy RXN wrapper with all three fixed-width counts. Raw MOL payloads
/// retain every supplied byte, including headers, unknown properties and CRLF.
/// Only a missing final LF is appended; new wrapper records use LF. Complete
/// V2000 or V3000 molecules are accepted, never projected, converted or regrouped.
pub fn write_v2000(header: [&str; 3], components: &[ComponentInput<'_>]) -> Result<Vec<u8>, Error> {
    if components.len() > MAX_COMPONENTS {
        return Err(Error::Resource("more than 1024 components"));
    }
    let mut byte_count = 18_usize; // 5 signature bytes + 3 header LFs + 10 counts bytes.
    let mut line_count = 5_usize;
    let mut counts = Counts::default();
    let mut totals = Totals::default();
    let mut last = 0;
    for (index, line) in header.iter().enumerate() {
        ascii(line.as_bytes())?;
        if line.len() > MAX_LINE_BYTES {
            return Err(Error::Resource("reaction header exceeds 4096 bytes"));
        }
        if line.contains(['\r', '\n']) {
            return Err(syntax(index + 2, "single-line reaction header"));
        }
        add_bytes(&mut byte_count, line.len())?;
    }
    for (index, component) in components.iter().enumerate() {
        if component.role.rank() < last {
            return Err(syntax(0, "ordered reactant/product/reagent inputs"));
        }
        last = component.role.rank();
        let count = match component.role {
            Role::Reactant => &mut counts.reactants,
            Role::Product => &mut counts.products,
            Role::Reagent => &mut counts.reagents,
        };
        *count = count
            .checked_add(1)
            .filter(|value| *value <= 999)
            .ok_or(Error::Resource(
                "more than 999 components in one legacy role",
            ))?;
        let text = ascii(component.mol_bytes)?;
        let stats = validate_mol(component.mol_bytes, index + 1)?;
        totals.add(&stats)?;
        let mut cursor = Cursor::new(text);
        let end = cursor.mol_end()?;
        if end != component.mol_bytes.len() {
            return Err(Error::Unsupported {
                line: 0,
                feature: "constituent padding after M END",
            });
        }
        add_lines(&mut line_count, cursor.line)?;
        add_lines(&mut line_count, 1)?;
        add_bytes(&mut byte_count, 5)?;
        add_bytes(&mut byte_count, component.mol_bytes.len())?;
        if !component.mol_bytes.ends_with(b"\n") {
            add_bytes(&mut byte_count, 1)?;
        }
    }
    let mut output = Vec::with_capacity(byte_count);
    output.extend_from_slice(b"$RXN\n");
    for line in header {
        output.extend_from_slice(line.as_bytes());
        output.push(b'\n');
    }
    output.extend_from_slice(
        format!(
            "{:>3}{:>3}{:>3}\n",
            counts.reactants, counts.products, counts.reagents
        )
        .as_bytes(),
    );
    for component in components {
        output.extend_from_slice(b"$MOL\n");
        output.extend_from_slice(component.mol_bytes);
        if !component.mol_bytes.ends_with(b"\n") {
            output.push(b'\n');
        }
    }
    Ok(output)
}
