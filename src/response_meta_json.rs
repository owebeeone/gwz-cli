pub(crate) fn response_meta_json(meta: &gwz_core::ResponseMeta) -> serde_json::Value {
    let mut value = serde_json::json!({
        "request_id": meta.request_id,
        "schema_version": meta.schema_version,
        "action": format!("{:?}", meta.action),
        "aggregate_status": format!("{:?}", meta.aggregate_status),
        "operation_id": meta.operation_id,
        "message": meta.message,
    });
    if let Some(rows) = &meta.transport {
        value["transport"] = rows.iter().map(transport_observation_json).collect();
    }
    value
}

pub(crate) fn transport_observation_json(
    row: &gwz_core::TransportObservation,
) -> serde_json::Value {
    let method = match row.credential_method {
        gwz_core::TransportCredentialMethod::Unknown => "unknown",
        gwz_core::TransportCredentialMethod::File => "file",
        gwz_core::TransportCredentialMethod::Agent => "agent",
        gwz_core::TransportCredentialMethod::Helper => "helper",
    };
    let source = match row.selection_source {
        gwz_core::TransportSelectionSource::Ambient => "ambient",
        gwz_core::TransportSelectionSource::InvocationRemote => "invocation_remote",
        gwz_core::TransportSelectionSource::InvocationDefault => "invocation_default",
        gwz_core::TransportSelectionSource::LocalConfiguration => "local_configuration",
    };
    let operation = match row.operation {
        gwz_core::TransportOperation::Clone => "clone",
        gwz_core::TransportOperation::Fetch => "fetch",
        gwz_core::TransportOperation::Push => "push",
        gwz_core::TransportOperation::ReadAdvertisement => "read_advertisement",
    };
    serde_json::json!({
        "repository_path": row.repository_path, "remote": row.remote, "operation": operation,
        "credential_method": method, "selection_source": source,
        "credential_offered": row.credential_offered, "authenticated": row.authenticated,
        "public_key_fingerprint": row.public_key_fingerprint,
    })
}

pub(crate) fn transport_human_lines(rows: &[gwz_core::TransportObservation]) -> Vec<String> {
    rows.iter()
        .filter(|row| {
            row.credential_method != gwz_core::TransportCredentialMethod::Unknown
                || row.credential_offered
        })
        .map(|row| {
            let value = transport_observation_json(row);
            let authenticated = match row.authenticated {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            };
            let mut line = format!(
                "{} {}: credential={} source={} offered={} authenticated={}",
                row.repository_path,
                row.remote,
                value["credential_method"].as_str().unwrap(),
                value["selection_source"].as_str().unwrap(),
                row.credential_offered,
                authenticated
            );
            if let Some(fingerprint) = &row.public_key_fingerprint {
                line.push_str(&format!(" fingerprint={fingerprint}"));
            }
            line
        })
        .collect()
}
