// Values are taken from https://internetcomputer.org/docs/current/developer-docs/gas-cost
// for a 13-node app subnet.
pub const HTTP_OCALL_BASE_COST: u128 = 49_140_000;
pub const HTTP_OCALL_REQ_BYTE_COST: u128 = 5_200;
pub const HTTP_OCALL_RESP_BYTE_COST: u128 = 10_400;

// TODO: should we pass cycles in http_x methods or we should have a default one?
// this is the cost of the request sent when starting the client
pub const DEFAULT_HTTP_OUTCALL_COST: u128 = HTTP_OCALL_BASE_COST
    + 5 * 1024 * HTTP_OCALL_REQ_BYTE_COST
    + 144 * 1024 * HTTP_OCALL_RESP_BYTE_COST;
