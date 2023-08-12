use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Node;
use kube::api::ResourceExt;

/// Helper methods for [`Node`]
pub trait NodeExt {
    /// Returns a map of the node's conditions in the form:
    ///
    /// ```json
    /// {
    ///     "MemoryPressure": "False",
    ///     "DiskPressure": "False"
    /// }
    fn status_conditions(&self) -> Option<BTreeMap<&String, &String>>;
    /// Returns a map of the node's addresses in the form:
    ///
    /// ```json
    /// {
    ///     "InternalIP": "192.168.0.1",
    ///     "Hostname": "kubernetes"
    /// }
    fn addresses(&self) -> Option<BTreeMap<&String, &String>>;
    /// Wrapper around [`ResourceExt::name_any`]
    fn name(&self) -> String;
    /// Wrapper around [`ResourceExt::labels`]
    fn labels(&self) -> &BTreeMap<String, String>;
    /// Whether this node is unschedulable or not
    fn unschedulable(&self) -> bool;
}

impl NodeExt for Node {
    /// Wrapper around [`ResourceExt::name_any`]
    fn name(&self) -> String {
        self.name_any()
    }

    /// Wrapper around [`ResourceExt::labels`]
    fn labels(&self) -> &BTreeMap<String, String> {
        ResourceExt::labels(self)
    }

    /// Whether this node is unschedulable or not
    fn unschedulable(&self) -> bool {
        if let Some(spec) = self.spec.as_ref() {
            return spec.unschedulable.unwrap_or(false);
        }

        false
    }

    fn status_conditions(&self) -> Option<BTreeMap<&String, &String>> {
        Some(
            self.status
                .as_ref()?
                .conditions
                .as_ref()?
                .iter()
                .map(|condition| (&condition.type_, &condition.status))
                .collect(),
        )
    }

    fn addresses(&self) -> Option<BTreeMap<&String, &String>> {
        Some(
            self.status
                .as_ref()?
                .addresses
                .as_ref()?
                .iter()
                .map(|address| (&address.type_, &address.address))
                .collect(),
        )
    }
}
