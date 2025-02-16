// src/protocol.rs

use nom::{
  number::complete::{be_u8, be_u16, be_u32},
  sequence::tuple,
  error::Error as NomError,
};

#[derive(Debug)]
pub struct AmqpFrame {
  pub frame_type: u8,
  pub channel: u16,
  pub payload: Vec<u8>,
}

/// Parses the AMQP header for AMQP 0.9.1.
/// 
/// The header is exactly 8 bytes: "AMQP\0\0\9\1"
pub fn parse_amqp_header(input: &[u8]) -> Result<(), &'static str> {
  let expected = b"AMQP\x00\x00\x09\x01";
  if input.len() < expected.len() {
      return Err("AMQP header too short");
  }
  if input.starts_with(expected) {
      Ok(())
  } else {
      Err("Invalid AMQP header")
  }
}

/// Parses an AMQP 0.9.1-like frame.
/// 
/// The expected frame layout is:
/// - 1 byte: frame type
/// - 2 bytes: channel (big-endian)
/// - 4 bytes: payload length (big-endian)
/// - `payload length` bytes: payload
/// - 1 byte: frame-end marker (must be 0xCE)
pub fn parse_amqp_frame(input: &[u8]) -> Result<AmqpFrame, &'static str> {
  // Check for minimum size: header (1 + 2 + 4 = 7 bytes) plus the frame-end marker
  if input.len() < 8 {
      return Err("Input too short for a valid frame");
  }
  
  let mut parser = tuple::<&[u8], (u8, u16, u32), NomError<&[u8]>, _>((be_u8, be_u16, be_u32));
  let (remainder, (frame_type, channel, payload_len)) = match parser(input) {
      Ok(res) => res,
      Err(_) => return Err("Failed to parse frame header"),
  };

  // Ensure the remainder contains the full payload and the frame-end marker.
  if remainder.len() < payload_len as usize + 1 {
      return Err("Not enough bytes for payload + frame-end");
  }

  let (payload, last_byte) = remainder.split_at(payload_len as usize);
  let frame_end = last_byte[0];
  if frame_end != 0xCE {
      return Err("Invalid frame-end marker, expected 0xCE");
  }

  Ok(AmqpFrame {
      frame_type,
      channel,
      payload: payload.to_vec(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_amqp_header_valid() {
      let header = b"AMQP\x00\x00\x09\x01";
      assert!(parse_amqp_header(header).is_ok());
  }

  #[test]
  fn test_parse_amqp_header_invalid() {
      let header = b"XYZ\x00\x00\x09\x01";
      assert!(parse_amqp_header(header).is_err());
  }

  #[test]
  fn test_parse_amqp_frame_success() {
      // Construct a valid frame:
      // - frame_type = 1
      // - channel = 1
      // - payload length = 3
      // - payload = [0xA, 0xB, 0xC]
      // - frame-end = 0xCE
      let mut frame_data = vec![];
      frame_data.push(1); // frame_type
      frame_data.extend_from_slice(&1u16.to_be_bytes()); // channel = 1
      frame_data.extend_from_slice(&3u32.to_be_bytes()); // payload length = 3
      frame_data.extend_from_slice(&[0xA, 0xB, 0xC]); // payload
      frame_data.push(0xCE); // frame-end

      let parsed = parse_amqp_frame(&frame_data).expect("Should parse successfully");
      assert_eq!(parsed.frame_type, 1);
      assert_eq!(parsed.channel, 1);
      assert_eq!(parsed.payload, vec![0xA, 0xB, 0xC]);
  }

  #[test]
  fn test_parse_amqp_frame_invalid_end_marker() {
      let mut frame_data = vec![];
      frame_data.push(1);
      frame_data.extend_from_slice(&1u16.to_be_bytes());
      frame_data.extend_from_slice(&1u32.to_be_bytes());
      // Insert payload byte
      frame_data.push(0xA); 
      // Wrong frame-end marker instead of 0xCE
      frame_data.push(0xAB);
      let parsed = parse_amqp_frame(&frame_data);
      assert!(parsed.is_err());
  }
}
