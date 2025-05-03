use parry3d_f64::shape::Compound;

#[derive(Clone)]
pub enum Object {
    Compound(Compound),
}
