use nikon_importer_core::ptp::{PtpOperation, PtpOperationCode, PtpResponse};

#[test]
fn encodes_operation_request_payload() {
    let operation = PtpOperation::new(PtpOperationCode::GetObjectInfo, 7, vec![123]);

    let payload = operation
        .encode_request_payload()
        .expect("operation should encode");

    assert_eq!(
        payload,
        vec![1, 0, 0, 0, 0x08, 0x10, 7, 0, 0, 0, 123, 0, 0, 0]
    );
}

#[test]
fn rejects_more_than_five_operation_params() {
    let operation = PtpOperation::new(
        PtpOperationCode::GetObjectHandles,
        1,
        vec![1, 2, 3, 4, 5, 6],
    );

    assert!(operation.encode_request_payload().is_err());
}

#[test]
fn decodes_response_payload() {
    let payload = vec![0x01, 0x20, 7, 0, 0, 0, 123, 0, 0, 0];

    let response = PtpResponse::decode_payload(&payload).expect("response should decode");

    assert_eq!(response.code.as_u16(), 0x2001);
    assert_eq!(response.transaction_id, 7);
    assert_eq!(response.params, vec![123]);
}
