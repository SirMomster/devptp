use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PortChanges {
    pub added: BTreeSet<u16>,
    pub removed: BTreeSet<u16>,
    pub ports_known: BTreeSet<u16>,
}

impl PortChanges {
    pub fn is_equal(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    pub fn compare_ports(previous: &BTreeSet<u16>, current: &BTreeSet<u16>) -> PortChanges {
        PortChanges {
            added: current.difference(previous).copied().collect(),
            removed: previous.difference(current).copied().collect(),
            ports_known: current.clone(),
        }
    }
}
