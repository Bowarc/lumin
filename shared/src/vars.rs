#[derive(
    serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, std::hash::Hash, Copy, Clone,
)]
pub enum VarId {
    MonitorList,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, std::hash::Hash)]
pub enum Var {
    MonitorList(Vec<crate::monitor::Monitor>),
    Other,
}

impl std::fmt::Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Var {
    pub fn monitor_list(&self) -> Option<&Vec<crate::monitor::Monitor>> {
        if let Var::MonitorList(list) = self {
            Some(list)
        } else {
            None
        }
    }
    pub fn monitor_list_mut(&mut self) -> Option<&mut Vec<crate::monitor::Monitor>> {
        if let Var::MonitorList(list) = self {
            Some(list)
        } else {
            None
        }
    }
}
