#[derive(Debug)]
pub struct RouteId(pub String);

impl RouteId {
    // TODO: ここもconst generics実装されたらtraitとしてひとまとめにしていいかも
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}