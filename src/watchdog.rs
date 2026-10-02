//! Watchdog state machine. Pure: feed it status observations and a clock, it returns the
//! actions a driver must take. The driver (`airlock watch`) polls; this file decides.
use crate::oneclick::SwapStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Refunded,
    Failed,
    /// Quote deadline passed with no deposit seen. Nothing was sent, nothing is held.
    Expired,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Seconds a funded part may sit non-terminal before the watchdog escalates.
    pub hold_timeout: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Done { part: usize, outcome: Outcome },
    Escalate { part: usize, reason: &'static str },
}

#[derive(Debug, Clone)]
struct Track {
    amount_zat: u64,
    deadline: u64,
    held_since: Option<u64>,
    escalated: bool,
    outcome: Option<Outcome>,
}

pub struct Tracker {
    cfg: Config,
    parts: Vec<Track>,
}

impl Tracker {
    /// `parts` is (amount_zat, quote deadline as unix seconds) per part.
    pub fn new(cfg: Config, parts: &[(u64, u64)]) -> Self {
        let parts = parts
            .iter()
            .map(|&(amount_zat, deadline)| Track { amount_zat, deadline, held_since: None, escalated: false, outcome: None })
            .collect();
        Self { cfg, parts }
    }

    pub fn observe(&mut self, part: usize, status: SwapStatus, now: u64) -> Vec<Action> {
        let cfg = self.cfg;
        let t = &mut self.parts[part];
        let mut out = Vec::new();
        if t.outcome.is_some() {
            return out;
        }
        let mut finish = |t: &mut Track, o: Outcome| {
            t.outcome = Some(o);
            out.push(Action::Done { part, outcome: o });
        };
        match status {
            SwapStatus::Success => finish(t, Outcome::Success),
            SwapStatus::Refunded => finish(t, Outcome::Refunded),
            SwapStatus::Failed => finish(t, Outcome::Failed),
            SwapStatus::PendingDeposit if now > t.deadline => finish(t, Outcome::Expired),
            SwapStatus::PendingDeposit => {}
            SwapStatus::KnownDepositTx | SwapStatus::Processing | SwapStatus::IncompleteDeposit => {
                let since = *t.held_since.get_or_insert(now);
                if status == SwapStatus::IncompleteDeposit && !t.escalated {
                    t.escalated = true;
                    out.push(Action::Escalate { part, reason: "deposit below the quoted amount, refund only at the quote deadline" });
                } else if now.saturating_sub(since) > cfg.hold_timeout && !t.escalated {
                    t.escalated = true;
                    out.push(Action::Escalate { part, reason: "funded part not terminal within the hold timeout" });
                }
            }
        }
        out
    }

    /// Value currently held by 1Click: funded parts that are not terminal.
    pub fn exposed_zat(&self) -> u64 {
        self.parts.iter().filter(|t| t.held_since.is_some() && t.outcome.is_none()).map(|t| t.amount_zat).sum()
    }

    /// Whether funding a part of `amount_zat` keeps the value held at once within `max_in_flight`.
    pub fn may_fund(&self, amount_zat: u64, max_in_flight: u64) -> bool {
        self.exposed_zat() + amount_zat <= max_in_flight
    }

    pub fn all_done(&self) -> bool {
        self.parts.iter().all(|t| t.outcome.is_some())
    }

    pub fn outcomes(&self) -> Vec<Option<Outcome>> {
        self.parts.iter().map(|t| t.outcome).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SwapStatus::*;

    const CFG: Config = Config { hold_timeout: 600 };

    fn tracker() -> Tracker {
        Tracker::new(CFG, &[(50_000_000, 10_000), (50_000_000, 10_000)])
    }

    #[test]
    fn happy_path_reaches_success_once() {
        let mut t = tracker();
        assert!(t.observe(0, PendingDeposit, 0).is_empty());
        assert!(t.observe(0, KnownDepositTx, 10).is_empty());
        assert!(t.observe(0, Processing, 20).is_empty());
        assert_eq!(t.observe(0, Success, 30), vec![Action::Done { part: 0, outcome: Outcome::Success }]);
        assert!(t.observe(0, Success, 40).is_empty(), "terminal is reported once");
    }

    #[test]
    fn stuck_processing_escalates_once_after_timeout() {
        let mut t = tracker();
        t.observe(0, Processing, 100);
        assert!(t.observe(0, Processing, 700).is_empty(), "exactly at the timeout is not past it");
        assert_eq!(t.observe(0, Processing, 701), vec![Action::Escalate { part: 0, reason: "funded part not terminal within the hold timeout" }]);
        assert!(t.observe(0, Processing, 9000).is_empty());
    }

    #[test]
    fn refund_and_failure_are_terminal_outcomes() {
        let mut t = tracker();
        assert_eq!(t.observe(0, Refunded, 5), vec![Action::Done { part: 0, outcome: Outcome::Refunded }]);
        assert_eq!(t.observe(1, Failed, 5), vec![Action::Done { part: 1, outcome: Outcome::Failed }]);
        assert!(t.all_done());
    }

    #[test]
    fn unfunded_part_expires_at_deadline() {
        let mut t = tracker();
        assert!(t.observe(0, PendingDeposit, 10_000).is_empty());
        assert_eq!(t.observe(0, PendingDeposit, 10_001), vec![Action::Done { part: 0, outcome: Outcome::Expired }]);
    }

    #[test]
    fn short_deposit_escalates_immediately() {
        let mut t = tracker();
        assert_eq!(t.observe(0, IncompleteDeposit, 5).len(), 1);
    }

    #[test]
    fn in_flight_cap_gates_the_next_funding() {
        let mut t = tracker();
        let cap = 60_000_000;
        assert!(t.may_fund(50_000_000, cap));
        t.observe(0, Processing, 1);
        assert_eq!(t.exposed_zat(), 50_000_000);
        assert!(!t.may_fund(50_000_000, cap), "second hold would put 1.0 ZEC in flight");
        t.observe(0, Success, 2);
        assert!(t.may_fund(50_000_000, cap), "released once the first part is terminal");
    }

    /// Statuses recorded from live /v0/status reads of public swaps on 2026-10-02 (13 non-success
    /// plus 5 success, tests/fixtures/recorded_statuses.json): every real status string must
    /// parse, and each terminal one must produce the matching outcome.
    #[test]
    fn recorded_live_statuses_drive_the_tracker() {
        let rec: Vec<serde_json::Value> = serde_json::from_str(include_str!("../tests/fixtures/recorded_statuses.json")).unwrap();
        let mut seen = std::collections::BTreeMap::new();
        for r in &rec {
            let st: SwapStatus = serde_json::from_value(r["status"].clone()).expect("real status string must parse");
            let mut t = Tracker::new(CFG, &[(1, u64::MAX)]);
            let acts = t.observe(0, st, 1);
            let want = match st {
                Success => Some(Outcome::Success),
                Refunded => Some(Outcome::Refunded),
                Failed => Some(Outcome::Failed),
                _ => None,
            };
            assert_eq!(acts.iter().find_map(|a| if let Action::Done { outcome, .. } = a { Some(*outcome) } else { None }), want, "{st:?}");
            *seen.entry(format!("{st:?}")).or_insert(0) += 1;
        }
        assert!(seen.contains_key("Refunded") && seen.contains_key("Processing") && seen.contains_key("Success"), "{seen:?}");
    }
}
