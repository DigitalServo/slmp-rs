/*
Request:
Header (Autoset) + Subheader + Access route + Data length + CPU timer + Data

Response with Data:
Header (Autoset) + Subheader + Access route + Data length + End code + Data

Request without Data:
Header (Autoset) + Subheader + Access route + Data length + End code

Error Response
Header (Autoset) + Subheader + Access route + Data length + End code + Error
*/

pub(crate) mod device_access;
pub(crate) mod unit_control;

const HEADER_BYTELEN: usize = 13;

const CPUTIMER_BYTELEN: usize = 2;
const COMMAND_BYTELEN: usize = 4;
const COMMAND_PREFIX_BYTELEN: usize = CPUTIMER_BYTELEN + COMMAND_BYTELEN;
