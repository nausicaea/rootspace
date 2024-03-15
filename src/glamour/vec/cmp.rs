#[cfg(test)]
impl<R> PartialEq<nalgebra::Vector4<R>> for super::Vec4<R>
    where
        R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::Vector4<R>) -> bool {
        self.w.eq(&rhs[3])
            && self.x.eq(&rhs[0])
            && self.y.eq(&rhs[1])
            && self.z.eq(&rhs[2])
    }
}

#[cfg(test)]
impl<R> PartialEq<cgmath::Vector4<R>> for super::Vec4<R>
    where
        R: PartialEq,
{
    fn eq(&self, rhs: &cgmath::Vector4<R>) -> bool {
        self.w.eq(&rhs.w)
            && self.x.eq(&rhs.x)
            && self.y.eq(&rhs.y)
            && self.z.eq(&rhs.z)
    }
}

