use nom::error::{VerboseError, VerboseErrorKind};

fn offset_fn(lhs: &[u8], rhs: &[u8]) -> usize {
    let fst = lhs.as_ptr();
    let snd = rhs.as_ptr();

    snd as usize - fst as usize
}

pub fn convert_error(input: &[u8], e: VerboseError<&[u8]>) -> String {
    use std::fmt::Write;
    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let offset = offset_fn(input, substring);

        if input.is_empty() {
            match kind {
                VerboseErrorKind::Char(c) => {
                    write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
                }
                VerboseErrorKind::Context(s) => {
                    write!(&mut result, "{}: in {}, got empty input\n\n", i, s)
                }
                VerboseErrorKind::Nom(e) => {
                    write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e)
                }
            }
        } else {
            let prefix = &input[..offset];

            // Count the number of newlines in the first `offset` bytes of input
            let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

            // Find the line that includes the subslice:
            // Find the *last* newline before the substring starts
            let line_begin = prefix
                .iter()
                .rev()
                .position(|&b| b == b'\n')
                .map(|pos| offset - pos)
                .unwrap_or(0);

            // Find the full line after that newline
            let line = &input[line_begin..]
                .split(|b| *b == 0x0a)
                .next()
                .unwrap_or(&input[line_begin..]);
            // Unstable: .trim_ascii_end();

            let line_str = String::from_utf8_lossy(line);

            // The (1-indexed) column number is the offset of our substring into that line
            let column_number = offset_fn(line, substring) + 1;

            match kind {
                VerboseErrorKind::Char(c) => {
                    if let Some(actual) = substring.iter().next() {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                            {line}\n\
                                 {caret:>column$}\n\
                                 expected '{expected}', found {actual}\n\n",
                            i = i,
                            line_number = line_number,
                            line = line_str,
                            caret = '^',
                            column = column_number,
                            expected = c,
                            actual = actual,
                        )
                    } else {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
                                {line}\n\
                                {caret:>column$}\n\
                                expected '{expected}', got end of input\n\n",
                            i = i,
                            line_number = line_number,
                            line = line_str,
                            caret = '^',
                            column = column_number,
                            expected = c,
                        )
                    }
                }
                VerboseErrorKind::Context(s) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {context}:\n\
                        {line}\n\
                        {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    context = s,
                    line = line_str,
                    caret = '^',
                    column = column_number,
                ),
                VerboseErrorKind::Nom(e) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {nom_err:?}:\n\
                        {line}\n\
                        {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    nom_err = e,
                    line = line_str,
                    caret = '^',
                    column = column_number,
                ),
            }
        }
        // Because `write!` to a `String` is infallible, this `unwrap` is fine.
        .unwrap();
    }

    result
}
