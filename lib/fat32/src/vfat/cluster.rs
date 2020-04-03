#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    // return the offset of the cluster(cluster 2 would have offset 0) 
    pub fn offset(&self) -> Option<u32> { 
        self.0.checked_sub(2)
    }

    pub fn index(&self) -> u32 {
        self.0
    }
}

// TODO: Implement any useful helper methods on `Cluster`.
