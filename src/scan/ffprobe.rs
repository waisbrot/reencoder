use regex::Regex;
use std::error::Error;
use std::str::FromStr;
use subprocess::Exec;
use subprocess::Redirection;

#[derive(Default)]
pub struct ProbedInfo {
    pub codec: Option<String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub bit_rate: Option<f32>,
}

type ProbeInfoResult = Result<ProbedInfo, Box<dyn Error>>;

trait ProbeResult {
    fn unpack_probe_result(&self) -> ProbeInfoResult;
}

impl ProbeResult for Option<serde_json::Value> {
    fn unpack_probe_result(&self) -> ProbeInfoResult {
        match self {
            None => Ok(ProbedInfo {
                ..Default::default()
            }),
            Some(value) => value.unpack_probe_result(),
        }
    }
}

impl ProbeResult for serde_json::Value {
    fn unpack_probe_result(&self) -> ProbeInfoResult {
        let bit_rate = self.get("bit_rate").parse_bit_rate();
        let codec = match self.get("codec_name") {
            None => None,
            Some(codec_name) => codec_name.as_str().map(|s| s.to_string()),
        };
        let height = option_downcast(self.get("height").unwrap().as_i64());
        let width = option_downcast(self.get("width").unwrap().as_i64());
        Ok(ProbedInfo {
            codec,
            height,
            width,
            bit_rate,
        })
    }
}

pub fn probe(path: &String) -> ProbeInfoResult {
    ffprobe_data(path).unpack_probe_result()
}

fn option_downcast(value: Option<i64>) -> Option<i32> {
    value.map(|n| n as i32)
}

trait HasBitRate {
    fn parse_bit_rate(&self) -> Option<f32>;
}

impl HasBitRate for Option<&serde_json::Value> {
    fn parse_bit_rate(&self) -> Option<f32> {
        match self {
            None => None,
            Some(json_value) => json_value.parse_bit_rate(),
        }
    }
}

impl HasBitRate for serde_json::Value {
    fn parse_bit_rate(&self) -> Option<f32> {
        match self.as_str() {
            None => None,
            Some(string) => string.parse_bit_rate(),
        }
    }
}

impl HasBitRate for str {
    fn parse_bit_rate(&self) -> Option<f32> {
        lazy_static! {
            static ref PATTERN: Regex = Regex::new(r"^\s*(\d*\.?\d+)\s*Kbit/s\s*$").unwrap();
        };
        match PATTERN.captures(self) {
            None => None,
            Some(captures) => {
                let kbps_string = &captures[0];
                match f32::from_str(kbps_string) {
                    Ok(kbps) => Some(kbps),
                    Err(e) => {
                        warn!("Got error {:?} while parsing Kbps", &e);
                        None
                    }
                }
            }
        }
    }
}

fn ffprobe_data(path: &String) -> Option<serde_json::Value> {
    trace!("ffprobe {}", &path);
    let captured = Exec::cmd("ffprobe")
        .arg("-show_streams")
        .arg("-loglevel")
        .arg("error")
        .arg("-print_format")
        .arg("json")
        .arg(path)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
        .unwrap();
    if captured.success() {
        let result: String = captured.stdout_str();
        trace!("ffprobe says {}", &result);
        let result_str: &str = result.as_str();
        let parsed: serde_json::Value = serde_json::from_str(result_str).unwrap();
        let streams: &Vec<serde_json::Value> = parsed["streams"].as_array().unwrap();
        let mut target_stream: Option<serde_json::Value> = None;
        for item in streams {
            if item["codec_type"] == "video" {
                target_stream = Some(item.to_owned());
                break;
            }
        }
        target_stream
    } else {
        warn!("ffprobe non-success: {}", &captured.stderr_str());
        None
    }
}
