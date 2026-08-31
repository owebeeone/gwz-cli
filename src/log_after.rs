pub(crate) const LOG_AFTER: &str = "\
Examples:
  gwz log
  gwz log --full --body
  gwz log -n 20 --author 'Ada <ada@example.com>'
  gwz log --since 2026-08-01T00:00:00Z
  gwz log main..topic -- src
  gwz --target mem_api log +release..HEAD
  gwz log --strict --tagged v0.11.1";
