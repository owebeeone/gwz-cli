use crate::*;

pub(crate) fn render_response(response: &CliResponse, output: OutputMode) -> String {
    // forall already streamed member output live; render only its trailing summary.
    if let Some(summary) = &response.summary {
        return summary.clone();
    }
    if let Some(listing) = &response.listing {
        return match output {
            OutputMode::Json | OutputMode::Jsonl => listing_json(listing).to_string(),
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
