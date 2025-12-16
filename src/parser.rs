use anyhow::{Result, anyhow};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Quota {
    pub filesystem: String,
    pub usage: f64,
    pub soft_quota: f64,
    pub hard_quota: f64,
    pub files: u64,
    pub billed_usage: f64,
    pub free_snaps: f64,
    pub custom_snaps: f64,
}

pub fn parse_quota_output(output: &str) -> Result<Vec<Quota>> {
    let mut lines = output.lines();
    // Stores the location of the metric values
    let mut header_indices: Vec<(String, usize)> = Vec::new();
    let mut results = Vec::new();

    // Locate the header first
    let header_line = lines
        .find(|line| line.trim().starts_with("Filesystem"))
        .ok_or_else(|| anyhow!("Header line not found"))?;

    // Locate the column
    let mut current_idx = 0;
    while current_idx < header_line.len() {
        if let Some(start_offset) = header_line[current_idx..].find(|c: char| !c.is_whitespace()) {
            let start = current_idx + start_offset;
            let end_offset = header_line[start..]
                .find(|c: char| c.is_whitespace())
                .unwrap_or(header_line[start..].len());
            let end = start + end_offset;

            let col_name = &header_line[start..end];
            header_indices.push((col_name.to_string(), start));

            current_idx = end;
        } else {
            break;
        }
    }

    // Assume that the numbers are directly below the columns
    for line in lines {
        let trimmed = line.trim();
        // This is where the trailing lines are dropped
        if trimmed.is_empty() || trimmed.starts_with('*') {
            continue;
        }

        let mut filesystem = String::new();
        let mut usage = 0.0;
        let mut soft_quota = 0.0;
        let mut hard_quota = 0.0;
        let mut files = 0;
        let mut billed_usage = 0.0;
        let mut free_snaps = 0.0;
        let mut custom_snaps = 0.0;

        for i in 0..header_indices.len() {
            let (col_name, start) = &header_indices[i];

            if *start >= line.len() {
                continue;
            }

            let end = if i + 1 < header_indices.len() {
                header_indices[i + 1].1
            } else {
                line.len()
            };

            let safe_end = std::cmp::min(end, line.len());
            let raw_val = &line[*start..safe_end];
            let val = raw_val.trim();

            if val.is_empty() {
                continue;
            }

            match col_name.as_str() {
                "Filesystem" => filesystem = val.to_string(),
                "Usage" => usage = f64::from_str(val).unwrap_or(0.0),
                "SoftQuota" => soft_quota = f64::from_str(val).unwrap_or(0.0),
                "HardQuota" => hard_quota = f64::from_str(val).unwrap_or(0.0),
                "Files" => files = u64::from_str(val).unwrap_or(0),
                "BilledUsage" => billed_usage = f64::from_str(val).unwrap_or(0.0),
                "FreeSnaps" => free_snaps = f64::from_str(val).unwrap_or(0.0),
                "CustomSnaps" => custom_snaps = f64::from_str(val).unwrap_or(0.0),
                _ => {}
            }
        }

        results.push(Quota {
            filesystem,
            usage,
            soft_quota,
            hard_quota,
            files,
            billed_usage,
            free_snaps,
            custom_snaps,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quota_output_decimal() {
        let output = r#"
Disk Quotas for User de4183

Filesystem      Usage           SoftQuota       HardQuota       Files           BilledUsage     FreeSnaps       CustomSnaps

data2           55.542          112             123.2           22054           85.979                          30.437


   *All figures reported in GB

   **BilledUsage is the sum of Usage and CustomSnaps
"#;

        let quotas = parse_quota_output(output).expect("Failed to parse");
        assert_eq!(quotas.len(), 1);
        let q = &quotas[0];

        assert_eq!(q.filesystem, "data2");
        assert_eq!(q.usage, 55.542);
        assert_eq!(q.soft_quota, 112.0);
        assert_eq!(q.hard_quota, 123.2);
        assert_eq!(q.files, 22054);
        assert_eq!(q.billed_usage, 85.979);
        assert_eq!(q.free_snaps, 0.0);
        assert_eq!(q.custom_snaps, 30.437);
    }

    #[test]
    fn test_parse_quota_output_integer_usage() {
        let output = r#"
Disk Quotas for User de4183

Filesystem      Usage         SoftQuota       HardQuota       Files           BilledUsage     FreeSnaps       CustomSnaps

data2           55            112             123.2           22054           85              10              30


   *All figures reported in GB

   **BilledUsage is the sum of Usage and CustomSnaps
"#;

        let quotas = parse_quota_output(output).expect("Failed to parse");
        assert_eq!(quotas.len(), 1);
        let q = &quotas[0];

        assert_eq!(q.filesystem, "data2");
        assert_eq!(q.usage, 55.0); // Expect 55.0 when input is "55"
        assert_eq!(q.soft_quota, 112.0);
        assert_eq!(q.hard_quota, 123.2);
        assert_eq!(q.files, 22054);
        assert_eq!(q.billed_usage, 85.0); // Expect 85.0 when input is "85"
        assert_eq!(q.free_snaps, 10.0); // Expect 10.0 when input is "10"
        assert_eq!(q.custom_snaps, 30.0); // Expect 30.0 when input is "30"
    }
}
