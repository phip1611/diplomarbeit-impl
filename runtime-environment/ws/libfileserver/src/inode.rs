/// I node.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub struct INode(u64);

impl INode {
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    pub const fn val(self) -> u64 {
        self.0
    }
}

impl<T> From<T> for INode
where
    T: Into<u64>,
{
    fn from(val: T) -> Self {
        INode::new(val.into())
    }
}
