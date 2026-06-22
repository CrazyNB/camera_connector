use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use camera_connector_core::{
    CameraConnectorService, ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier,
    ModelEvaluatorKind, PreviewSample, ProjectEvaluationSettings, StoredObjectLocation,
    TransferRecord, TransferStatus,
};

pub(super) struct TestModelServer {
    address: String,
    request_rx: mpsc::Receiver<String>,
}

impl TestModelServer {
    pub(super) fn start(response_body: &'static str) -> Self {
        Self::start_owned(response_body.to_string())
    }

    pub(super) fn start_owned(response_body: String) -> Self {
        Self::start_sequence_owned(vec![response_body])
    }

    pub(super) fn start_sequence_owned(response_bodies: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("server should bind");
        let address = listener.local_addr().expect("server addr").to_string();
        let (request_tx, request_rx) = mpsc::channel();
        thread::spawn(move || {
            for response_body in response_bodies {
                let (mut stream, _) = listener.accept().expect("server should accept request");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("read timeout should set");
                let mut request_bytes = Vec::new();
                let mut buffer = [0_u8; 4096];
                loop {
                    let size = stream.read(&mut buffer).expect("request should read");
                    if size == 0 {
                        break;
                    }
                    request_bytes.extend_from_slice(&buffer[..size]);
                    if complete_http_request_len(&request_bytes)
                        .is_some_and(|expected| request_bytes.len() >= expected)
                    {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&request_bytes).to_string();
                request_tx.send(request).expect("request should send");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body,
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("response should write");
            }
        });
        Self {
            address,
            request_rx,
        }
    }

    pub(super) fn base_url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub(super) fn received_request(&self) -> Option<String> {
        self.request_rx.recv_timeout(Duration::from_secs(2)).ok()
    }

    pub(super) fn received_requests(&self, count: usize) -> Vec<String> {
        (0..count)
            .filter_map(|_| self.request_rx.recv_timeout(Duration::from_secs(2)).ok())
            .collect()
    }
}

pub(super) fn complete_http_request_len(buffer: &[u8]) -> Option<usize> {
    let header_end = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]).to_ascii_lowercase();
    let content_length = headers.lines().find_map(|line| {
        line.strip_prefix("content-length:")
            .and_then(|value| value.trim().parse::<usize>().ok())
    })?;
    Some(header_end + 4 + content_length)
}

pub(super) fn enable_upload_model_evaluation(service: &CameraConnectorService, project_id: &str) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.auto_evaluate_on_upload = true;
    settings.prompt_pack_id = Some("general-default".to_string());
    service
        .save_project_evaluation_settings(ProjectEvaluationSettings { ..settings })
        .expect("settings should save");
}

pub(super) fn select_model_provider(
    service: &CameraConnectorService,
    project_id: &str,
    settings_id: &str,
) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.model_provider_settings_id = Some(settings_id.to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
}

pub(super) fn select_prompt_pack(
    service: &CameraConnectorService,
    project_id: &str,
    prompt_pack_id: &str,
) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(prompt_pack_id.to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
}

pub(super) fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .unwrap_or(original_path)
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 1024,
        username: Some("camera".to_string()),
        remote_addr: Some("127.0.0.1".to_string()),
        source_name: Some("HTTP Camera".to_string()),
        started_at_ms: at_ms,
        completed_at_ms: Some(at_ms),
        error: None,
    }
}

pub(super) fn balanced_preview_sample() -> PreviewSample {
    PreviewSample {
        width: 4,
        height: 4,
        luma: vec![
            20, 40, 70, 90, 45, 80, 120, 145, 60, 110, 160, 205, 80, 130, 190, 235,
        ],
        red: None,
        green: None,
        blue: None,
        preview_source: Some("jpeg".to_string()),
    }
}

pub(super) fn model_evaluation(
    project_id: &str,
    asset_group_id: &str,
    evaluator_version: &str,
    score: i64,
    summary: &str,
) -> ModelEvaluation {
    ModelEvaluation {
        evaluation_id: format!("evaluation-{asset_group_id}-{evaluator_version}"),
        run_id: format!("run-{asset_group_id}-{evaluator_version}"),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        evaluator_kind: ModelEvaluatorKind::LlmVlm,
        evaluator_version: evaluator_version.to_string(),
        status: ModelEvaluationStatus::Ready,
        score,
        tier: ModelEvaluationTier::from_score(score),
        selectable: score >= 50,
        summary: summary.to_string(),
        strengths: Vec::new(),
        weaknesses: Vec::new(),
        technical_warnings: Vec::new(),
        prompt_pack_id: None,
        prompt_pack_version: None,
        prompt_hash: None,
        created_at_ms: 1_500,
        updated_at_ms: 1_500,
    }
}
