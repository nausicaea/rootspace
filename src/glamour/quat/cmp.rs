#[cfg(test)]
impl<R> PartialEq<nalgebra::Quaternion<R>> for super::Quat<R> 
where
    R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::Quaternion<R>) -> bool {
        self.w.eq(&rhs.coords.w)
            && self.i.eq(&rhs.coords.x)
            && self.j.eq(&rhs.coords.y)
            && self.k.eq(&rhs.coords.z)
    }
}

#[cfg(test)]
impl<R> PartialEq<cgmath::Quaternion<R>> for super::Quat<R> 
where
    R: PartialEq,
{
    fn eq(&self, rhs: &cgmath::Quaternion<R>) -> bool {
        self.w.eq(&rhs.s) 
            && self.i.eq(&rhs.v.x) 
            && self.j.eq(&rhs.v.y) 
            && self.k.eq(&rhs.v.z)
    }
}

