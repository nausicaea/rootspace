use num_traits::Zero;

use super::Mat;

impl<R, const I: usize, const J: usize> serde::ser::Serialize for Mat<R, I, J>
where
    R: serde::ser::Serialize,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut state = ser.serialize_seq(Some(I * J))?;
        for row in &self.0 {
            for cell in row {
                state.serialize_element(cell)?;
            }
        }
        state.end()
    }
}

impl<'de, R, const I: usize, const J: usize> serde::de::Deserialize<'de> for Mat<R, I, J>
where
    R: Zero + Copy + serde::de::Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MatVisitor<R, const I: usize, const J: usize>(std::marker::PhantomData<[[R; J]; I]>);

        impl<R, const I: usize, const J: usize> Default for MatVisitor<R, I, J> {
            fn default() -> Self {
                MatVisitor(std::marker::PhantomData::default())
            }
        }

        impl<'v, R, const I: usize, const J: usize> serde::de::Visitor<'v> for MatVisitor<R, I, J>
        where
            R: Zero + Copy + serde::de::Deserialize<'v>,
        {
            type Value = Mat<R, I, J>;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "a sequence with {} elements", I * J)
            }

            fn visit_seq<A>(self, mut acc: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'v>,
            {
                use serde::de::Error;

                let mut mat: Mat<R, I, J> = Mat::zero();

                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = acc.next_element()?.ok_or_else(|| {
                            Error::custom(format!("unexpected end of the serialized sequence of length {}", I * J))
                        })?;
                    }
                }

                Ok(mat)
            }
        }

        de.deserialize_seq(MatVisitor::default())
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn mat_implements_serde() {
        let a: Mat<f32, 2, 2> = Mat::identity();

        assert_tokens(
            &a,
            &[
                Token::Seq { len: Some(4) },
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::SeqEnd,
            ],
        );
    }
}
