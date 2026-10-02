//! Quote profile: the fields of a quote that `/v0/status` shows to anyone, reduced to the shape
//! an observer can group swaps by. `matches` counts how many swaps in a measured sample share it:
//! the size of the crowd a part hides in.
use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub referral: String,
    pub fees: String,
    pub slippage_bps: u64,
    pub deadline_min: i64,
    pub refund: String,
}

#[derive(Debug, Deserialize)]
struct Row {
    referral: String,
    fees: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u64,
    #[serde(rename = "deadlineMin")]
    deadline_min: i64,
    refund: String,
    count: u64,
}

#[derive(Debug, Deserialize)]
struct Sample {
    swaps: u64,
}

#[derive(Debug, Deserialize)]
struct File {
    sample: Sample,
    profiles: Vec<Row>,
}

pub struct Reference(File);

impl Reference {
    pub fn load(path: &Path) -> Result<Self> {
        Ok(Self(serde_json::from_slice(&std::fs::read(path)?)?))
    }

    pub fn sample_size(&self) -> u64 {
        self.0.sample.swaps
    }

    /// Swaps in the sample with exactly this profile.
    pub fn matches(&self, p: &Profile) -> u64 {
        self.0
            .profiles
            .iter()
            .filter(|r| r.referral == p.referral && r.fees == p.fees && r.slippage_bps == p.slippage_bps && r.deadline_min == p.deadline_min && r.refund == p.refund)
            .map(|r| r.count)
            .sum()
    }
}

impl Profile {
    /// Reads the profile from a `/v0/status` body. None when the body lacks the quote request.
    pub fn from_status(raw: &Value) -> Option<Self> {
        let q = &raw["quoteResponse"]["quoteRequest"];
        let ts = crate::oneclick::parse_rfc3339(raw["quoteResponse"]["timestamp"].as_str()?)? as i64;
        let dl = crate::oneclick::parse_rfc3339(q["deadline"].as_str()?)? as i64;
        let fees = q["appFees"]
            .as_array()
            .map(|a| a.iter().map(|f| format!("{}:{}", f["recipient"].as_str().unwrap_or("").chars().take(6).collect::<String>(), f["fee"])).collect::<Vec<_>>().join(","))
            .unwrap_or_default();
        let r = q["refundTo"].as_str()?;
        let refund = if r.starts_with("u1") { format!("u{}", r.len()) } else if r.starts_with("t1") { "t".into() } else { "other".into() };
        Some(Self {
            referral: q["referral"].as_str().unwrap_or("").into(),
            fees,
            slippage_bps: q["slippageTolerance"].as_u64()?,
            deadline_min: (dl - ts + 30).div_euclid(60),
            refund,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        let mut v: Value = serde_json::from_str(include_str!("../tests/fixtures/status_pending.json")).unwrap();
        v["quoteResponse"]["quoteRequest"]["slippageTolerance"] = 200.into();
        v
    }

    #[test]
    fn profile_reads_the_public_fields() {
        let p = Profile::from_status(&fixture()).unwrap();
        assert_eq!((p.referral.as_str(), p.slippage_bps, p.deadline_min), ("", 200, 120));
        assert_eq!(p.fees, "5880ad:20");
        assert_eq!(p.refund, "t");
    }

    #[test]
    fn matches_counts_exact_profiles_only() {
        let r = Reference(serde_json::from_str(r#"{"sample":{"swaps":10},"profiles":[
            {"referral":"","fees":"5880ad:20","slippageBps":200,"deadlineMin":120,"refund":"t","count":4},
            {"referral":"","fees":"5880ad:20","slippageBps":200,"deadlineMin":60,"refund":"t","count":6}]}"#).unwrap());
        let p = Profile::from_status(&fixture()).unwrap();
        assert_eq!((r.sample_size(), r.matches(&p)), (10, 4));
        let mut other = p.clone();
        other.referral = "zodl".into();
        assert_eq!(r.matches(&other), 0);
    }
}
