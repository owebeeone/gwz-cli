use crate::*;

pub(crate) fn render_response(response: &CliResponse, output: OutputMode) -> String {
    let mut rendered = render_response_inner(response, output);
    if output == OutputMode::Human {
        for line in transport_human_lines(
            response
                .envelope
                .meta
                .transport
                .as_deref()
                .unwrap_or_default(),
        ) {
            rendered.push('\n');
            rendered.push_str(&line);
        }
    }
    rendered
}

fn render_response_inner(response: &CliResponse, output: OutputMode) -> String {
    // forall already streamed member output live; render only its trailing summary.
    if let Some(summary) = &response.summary {
        return summary.clone();
    }
    if let Some(listing) = &response.listing {
        if matches!(listing, ArtifactListing::Identities(_)) {
            return match output {
                OutputMode::Json | OutputMode::Jsonl => {
                    let mut value = response_json(response);
                    value["identities"] = listing_json(listing)["entries"].clone();
                    value.to_string()
                }
                OutputMode::Human | OutputMode::Porcelain => {
                    let mut text = render_listing_text(listing);
                    for row in &response.envelope.members {
                        if row.status == gwz_core::MemberStatus::Planned {
                            text.push_str(&format!(
                                "\n{}: planned (configuration unchanged)",
                                row.member_id
                            ));
                        }
                        if let Some(error) = &row.error {
                            text.push_str(&format!("\n{}: {}", row.member_id, error.message));
                        }
                    }
                    text
                }
            };
        }
        return match output {
            OutputMode::Json | OutputMode::Jsonl => {
                let mut value = listing_json(listing);
                if let Some(rows) = &response.envelope.meta.transport {
                    value["transport"] = rows.iter().map(transport_observation_json).collect();
                }
                value.to_string()
            }
            OutputMode::Human | OutputMode::Porcelain => render_listing_text(listing),
        };
    }
    match output {
        OutputMode::Human => render_human_response(response),
        OutputMode::Json => response_json(response).to_string(),
        OutputMode::Jsonl => render_jsonl_stream(response, &[], None),
        OutputMode::Porcelain => render_porcelain_response(response),
    }
}

pub(crate) fn exit_code_for_response(response: &gwz_core::ResponseEnvelope) -> i32 {
    match response.meta.aggregate_status {
        gwz_core::AggregateStatus::Accepted
        | gwz_core::AggregateStatus::Ok
        | gwz_core::AggregateStatus::Noop
        // F5/AD3: a dirty workspace is the normal resting state (like `git status`) — exit 0.
        | gwz_core::AggregateStatus::Dirty => 0,
        gwz_core::AggregateStatus::Rejected => 2,
        // A conflict needs developer action (resolve + continue) — exit non-zero, like `git rebase`.
        gwz_core::AggregateStatus::Partial
        | gwz_core::AggregateStatus::Failed
        | gwz_core::AggregateStatus::Conflicted => 1,
    }
}
