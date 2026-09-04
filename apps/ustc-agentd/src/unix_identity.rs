//! plan_ref: docs/plan/modules/90-infrastructure-operations.md#security-boundary
//! Linux process-identity queries shared by durable and secret-file adapters.

/// Returns the effective UID reported by the kernel process-status interface.
pub(crate) fn effective_uid() -> Result<u32, String> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("process status unavailable: {error}"))?;
    parse_effective_uid(&status)
}

fn parse_effective_uid(status: &str) -> Result<u32, String> {
    let uid_line = status
        .lines()
        .find(|line| line.starts_with("Uid:"))
        .ok_or_else(|| "process status has no Uid field".to_owned())?;
    let mut fields = uid_line.split_ascii_whitespace();
    if fields.next() != Some("Uid:") {
        return Err("process status Uid field is malformed".to_owned());
    }
    let _real_uid = fields
        .next()
        .ok_or_else(|| "process status real uid is missing".to_owned())?;
    let effective_uid = fields
        .next()
        .ok_or_else(|| "process status effective uid is missing".to_owned())?;
    effective_uid
        .parse::<u32>()
        .map_err(|_| "process status effective uid is invalid".to_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn parser_uses_the_effective_not_procfs_inode_owner_uid() {
        let status = "Name:\ttest\nUid:\t1000\t65532\t65532\t65532\n";
        assert_eq!(super::parse_effective_uid(status), Ok(65_532));
    }
}
