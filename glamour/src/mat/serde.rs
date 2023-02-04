use super::Mat4;

impl<R> serde::ser::Serialize for Mat4<R>
where
    R: serde::ser::Serialize,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut state = ser.serialize_seq(Some(16))?;
        for row in &self.0 {
            for cell in row {
                state.serialize_element(cell)?;
            }
        }
        state.end()
    }
}

impl<'de, R> serde::de::Deserialize<'de> for Mat4<R>
where
    Self: crate::Zero,
    R: Copy + serde::de::Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MatVisitor<R>(std::marker::PhantomData<[[R; 4]; 4]>);

        impl<R> Default for MatVisitor<R> {
            fn default() -> Self {
                MatVisitor(std::marker::PhantomData::default())
            }
        }

        impl<'v, R> serde::de::Visitor<'v> for MatVisitor<R>
        where
            Mat4<R>: crate::Zero,
            R: Copy + serde::de::Deserialize<'v>,
        {
            type Value = Mat4<R>;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "a sequence with {} elements", 16)
            }

            fn visit_seq<A>(self, mut acc: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'v>,
            {
                use crate::Zero;
                use serde::de::Error;

                let mut mat: Mat4<R> = Mat4::zero();

                for i in 0..4 {
                    for j in 0..4 {
                        mat[(i, j)] = acc.next_element()?.ok_or_else(|| {
                            Error::custom(format!("unexpected end of the serialized sequence of length {}", 16))
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
        let a: Mat4<f32> = Mat4::identity();

        assert_tokens(
            &a,
            &[
                Token::Seq { len: Some(16) },
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::SeqEnd,
            ],
        );
    }
}
