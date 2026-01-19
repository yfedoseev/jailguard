# Security Policy

## Reporting Security Vulnerabilities

JailGuard takes security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Do NOT** create a public GitHub issue for security vulnerabilities. Instead:

1. **Email:** Send details to: [security@jailguard.dev](mailto:security@jailguard.dev)
   - Subject: `[SECURITY] Vulnerability in JailGuard`
   - Include: Description, steps to reproduce, impact assessment

2. **GitHub Security Advisory:** (Alternative)
   - Go to: https://github.com/yfedoseev/jailguard/security/advisories
   - Click "Report a vulnerability"
   - Provide details in the form

### What to Include

- **Description:** Clear explanation of the vulnerability
- **Steps to reproduce:** Detailed reproduction instructions
- **Impact:** Security impact (data exposure, DoS, code execution, etc.)
- **Affected versions:** Which versions of JailGuard are affected
- **Proof of concept:** If possible (but not required)
- **Your contact:** How to reach you for follow-up questions

### Response Timeline

We aim to:
- Acknowledge receipt within **24 hours**
- Provide initial assessment within **3 business days**
- Release patch within **7-14 days** (depending on complexity)
- Credit reporter in security advisory (unless requested anonymously)

## Security Practices

### Dependency Management

JailGuard maintains a strong security posture through:

- **Cargo-deny:** Blocks unknown registries and Git sources
  ```bash
  cargo deny check
  ```

- **Security audit:** Checks for known vulnerabilities
  ```bash
  cargo audit
  ```

- **Automated CI/CD:** Security checks run on every PR
  - Format checking
  - Linting with Clippy
  - Dependency audit
  - Multi-platform testing

### Known Security Issues

#### RUSTSEC-2026-0002: lru IterMut Soundness Issue

- **Package:** lru crate
- **Severity:** Low (non-critical)
- **Status:** Acknowledged, upgrade planned
- **Impact:** Only affects embedding cache iteration (non-critical path)
- **Fix:** Planned upgrade to lru >=0.16.3 in v1.1.1 patch release
- **References:**
  - https://github.com/jeromefroe/lru-rs/pull/224
  - RUSTSEC-2026-0002

#### RUSTSEC-2024-0436: Paste Crate Unmaintained

- **Package:** paste (transitive dependency via burn)
- **Severity:** Low (not a security vulnerability)
- **Status:** Archived by maintainer (dtolnay)
- **Impact:** No security risk, just unmaintained
- **Note:** Required by burn framework dependency chain
- **Resolution:** Monitored for future vulnerabilities

#### RUSTSEC-2025-0141: Bincode Unmaintained

- **Package:** bincode
- **Severity:** Low (not a security vulnerability)
- **Status:** Unmaintained but stable
- **Impact:** No known vulnerabilities
- **Note:** Used by burn for serialization
- **Plan:** Monitor for burnpack migration

### Unsafe Code

JailGuard minimizes use of unsafe code. All unsafe code is:

- **Justified:** Documented with comments explaining necessity
- **Audited:** Reviewed during code review
- **Tested:** Covered by test suite
- **Minimal:** Only 4 uses of unsafe blocks in the codebase

To find unsafe code:
```bash
cargo clippy --all-targets -- -W unsafe_code
```

### Code Review

- **All changes** go through GitHub pull requests
- **CI checks** must pass before merge
- **Code review** by maintainers before approval
- **Security audit** integrated into CI

## Best Practices for Users

### Safe Configuration

**Strict Mode** (recommended for security-critical applications):
```rust
let config = JailGuardConfig {
    block_threshold: 0.5,  // Lower threshold = more sensitive
    strict_mode: true,      // Block on any layer detection
    ..Default::default()
};
```

### Confidence Thresholding

- **Always use confidence scores** when available
- **Don't rely on single layer** - use ensemble approach
- **Calibrated confidence** - Use ECE metric for reliability

### Regular Updates

- **Keep JailGuard updated** - Security patches released promptly
- **Monitor dependencies** - Run `cargo audit` regularly
- **Check advisories** - Subscribe to security advisories

## Security Contact

- **Email:** [security@jailguard.dev](mailto:security@jailguard.dev)
- **GitHub:** [@yfedoseev](https://github.com/yfedoseev)
- **Response target:** 24-72 hours

## Responsible Disclosure

We follow responsible disclosure practices:

1. ✅ **Report privately** - Don't disclose publicly until fixed
2. ✅ **Allow time** - Give us time to patch before disclosure
3. ✅ **Coordinate timeline** - Work with us on disclosure date
4. ✅ **Credit you** - We'll acknowledge your discovery (if desired)
5. ✅ **No retaliation** - No legal action for good-faith reports

## Security Testing

### Running Security Checks

```bash
# Security audit for known vulnerabilities
cargo audit

# Dependency checking and license compliance
cargo deny check

# Unsafe code analysis
cargo clippy --all-targets -- -W unsafe_code

# Full security checks (as in CI)
./scripts/security_check.sh  # if available
```

### Testing JailGuard Securely

When testing JailGuard:

- **Use controlled inputs** - Don't test with real data at scale
- **Monitor resource usage** - Watch for DoS vulnerabilities
- **Report issues responsibly** - Don't publish exploits
- **Get permission** - Ensure you have authorization for security testing

## Vulnerability Disclosure

When we discover or are notified of vulnerabilities:

1. **Assess severity** - Evaluate impact and CVSS score
2. **Create fix** - Develop patch and tests
3. **Internal review** - Verify fix is correct and complete
4. **Release patch** - Publish fixed version
5. **Publish advisory** - Post security advisory with details
6. **Communicate** - Notify users of available updates

## Security Advisories

Published security advisories are available at:
- https://github.com/yfedoseev/jailguard/security/advisories
- https://nvd.nist.gov (for CVE-tracked vulnerabilities)

## Security Roadmap

Future security improvements:

- [ ] Fuzz testing integration
- [ ] SBOM (Software Bill of Materials) generation
- [ ] Regular security audits
- [ ] Enhanced threat modeling
- [ ] Security training for contributors

## Additional Resources

- **OWASP Top 10:** https://owasp.org/www-project-top-ten/
- **Cargo security guide:** https://doc.rust-lang.org/cargo/
- **Rust security practices:** https://anssi-fr.github.io/rust-guide/

---

## Acknowledgments

Security researchers who have responsibly disclosed vulnerabilities:
(None currently - be the first!)

---

**Last updated:** 2026-01-18
