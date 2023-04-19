pub struct DvarMgr {
    loaded: std::collections::HashMap<shared::daemon::vars::DVarId, crate::request::Request>,
}
