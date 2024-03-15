#[cfg(test)]
impl<R> PartialEq<nalgebra::Matrix4<R>> for super::Mat4<R>
where
    R: PartialEq,
{
    fn eq(&self, other: &nalgebra::Matrix4<R>) -> bool {
        <[[R; 4]; 4] as PartialEq<[[R; 4]; 4]>>::eq(&self.0, &other.data.0)
    }
}

#[cfg(test)]
impl<R> PartialEq<cgmath::Matrix4<R>> for super::Mat4<R>
where
    R: PartialEq,
{
    fn eq(&self, other: &cgmath::Matrix4<R>) -> bool {
        <[[R; 4]; 4] as PartialEq<[[R; 4]; 4]>>::eq(&self.0, other.as_ref())
    }
}
