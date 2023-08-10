use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Pod;
use kube::api::ResourceExt;

/// Helper methods for [`Node`]
pub trait PodExt {
    /// The IP address of the pod within the cluster
    fn ip_addr(&self) -> Option<&String>;
    /// The phase the pod is currently in
    fn phase(&self) -> Option<&String>;
    /// Wrapper around [`ResourceExt::namespace`]
    fn namespace(&self) -> Option<String>;
    /// Wrapper around [`ResourceExt::name_any`]
    fn name(&self) -> String;
    /// Wrapper around [`ResourceExt::labels`]
    fn labels(&self) -> &BTreeMap<String, String>;
}

impl PodExt for Pod {
    fn name(&self) -> String {
        self.name_any()
    }

    /// Wrapper around [`ResourceExt::labels`]
    fn labels(&self) -> &BTreeMap<String, String> {
        ResourceExt::labels(self)
    }

    fn ip_addr(&self) -> Option<&String> {
        Some(self.status.as_ref()?.pod_ip.as_ref()?)
    }

    fn phase(&self) -> Option<&String> {
        Some(self.status.as_ref()?.phase.as_ref()?)
    }

    fn namespace(&self) -> Option<String> {
        ResourceExt::namespace(self)
    }
}
