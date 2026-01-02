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

const COMMAND_BYTELEN: usize = 4;
