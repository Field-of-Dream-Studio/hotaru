# Security Policy

Hotaru is a pre-1.0 experimental framework. This policy covers the main
repository, its published crates, official examples, templates, and workflows.
Hotaru MQTT uses the separate reporting route below.

1. **Supported versions**

   - Only the latest `0.8.x` patch is eligible for security fixes. Users of
     earlier releases should upgrade.
   - Reports about `master`, release candidates, and snapshots are welcome,
     but these development versions have no support guarantee.

2. **Report privately**

   - Do not open a public issue for an undisclosed vulnerability.
   - For Hotaru, use the repository's
     [Security page](https://github.com/Field-of-Dream-Studio/hotaru/security)
     if private reporting is enabled, or email
     [redstone@fds.moe](mailto:redstone@fds.moe) with the subject
     `[SECURITY] Hotaru: short description`.
   - For Hotaru MQTT, use its
     [Security page](https://github.com/Field-of-Dream-Studio/hotaru_mqtt/security)
     if private reporting is enabled, or email
     [jerrysu@fds.moe](mailto:jerrysu@fds.moe).
   - Include the affected crate, version or commit, features, reproduction or
     raw input, impact, required conditions, mitigation, and whether the issue
     is already public. Remove credentials, personal data, and unrelated
     secrets.

3. **Handling and disclosure**

   - We aim to acknowledge reports within three business days. Remediation
     time depends on severity, reproducibility, and release coordination.
   - Coordinate disclosure until a fix or practical mitigation is available.
     We may publish an advisory, patch, and upgrade guidance. Reporters are
     credited unless they request anonymity.
