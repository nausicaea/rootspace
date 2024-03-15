#[cfg(test)]
impl<R> PartialEq<nalgebra::Vector4<R>> for super::Vec4<R>
    where
        R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::Vector4<R>) -> bool {
        self.w.eq(&rhs.w)
            && self.x.eq(&rhs.x)
            && self.y.eq(&rhs.y)
            && self.z.eq(&rhs.z)
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

