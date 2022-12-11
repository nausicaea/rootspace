use nom::{bytes::complete::tag, IResult};

const PLY: &'static [u8] = b"ply";

pub fn ply(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PLY)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_detects_ply_byte_sequence() {
        let input = b"ply\nformat ascii 1.0\nend_header\n";
        let rest = b"\nformat ascii 1.0\nend_header\n";

        let r = ply(input.as_slice());

        assert_eq!(r, Ok((rest.as_slice(), PLY)));
    }
}
