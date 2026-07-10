pub(crate) const REPO_ATTACH_LONG: &str = "\
Reactivate an inactive repository designation in the current workspace while
preserving its member and source identities.

Attach requires the historical member id and retained checkout. Every
snapshot or marker commit recorded for that member must exist in the checkout.
If no historical evidence exists, explicit attach proceeds with a warning.";
