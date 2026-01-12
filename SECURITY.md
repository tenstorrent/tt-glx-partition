# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

As this project is in early development, we currently support only the latest release. Once the project matures, we will extend support to multiple versions.

## Reporting a Vulnerability

The TT-GLX-PARTITION team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report a Security Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by emailing the security team at:

**[security@tenstorrent.com]** (or the appropriate security contact for your organization)

Include the following information in your report:

- **Type of vulnerability** (e.g., buffer overflow, privilege escalation, etc.)
- **Full paths of source file(s)** related to the manifestation of the vulnerability
- **Location of the affected source code** (tag/branch/commit or direct URL)
- **Step-by-step instructions** to reproduce the issue
- **Proof-of-concept or exploit code** (if possible)
- **Impact of the vulnerability** including how an attacker might exploit it
- **Any potential mitigations** you've identified

### What to Expect

After submitting a vulnerability report, you can expect:

1. **Acknowledgment**: We will acknowledge receipt of your vulnerability report within 48 hours.

2. **Initial Assessment**: We will perform an initial assessment and provide an estimated timeline for a fix within 5 business days.

3. **Updates**: We will keep you informed of our progress toward resolving the vulnerability.

4. **Disclosure Timeline**: We aim to resolve critical vulnerabilities within 90 days. We will work with you on a reasonable disclosure timeline.

5. **Credit**: If you wish, we will publicly acknowledge your responsible disclosure once the vulnerability is resolved (unless you prefer to remain anonymous).

### Security Considerations for This Project

This project interacts directly with hardware devices at a low level:

- **Physical Hardware Access**: This tool requires direct access to PCIe devices and should only be run on trusted systems with appropriate access controls.

- **Root/Administrator Privileges**: Some operations may require elevated privileges. Always review code before running with elevated permissions.

- **Hardware State Modification**: This tool modifies the connectivity state of physical hardware. Improper use could affect system availability.

- **No Remote Execution**: This tool is designed for local execution only and should not be exposed to remote or untrusted networks.

### Security Best Practices

When using this tool:

1. **Verify Source**: Only run code from trusted sources. Review any changes before execution.

2. **Limit Access**: Restrict access to this tool to authorized personnel only.

3. **Audit Usage**: Log and monitor usage of this tool in production environments.

4. **Backup Configurations**: Always backup your current configuration before making changes.

5. **Test Changes**: Test configuration changes in a development environment when possible.

6. **Update Regularly**: Keep the tool updated to the latest version to receive security patches.

## Vulnerability Disclosure Policy

When we receive a security vulnerability report, we will:

1. Confirm the vulnerability and determine its impact
2. Develop and test a fix
3. Release a patched version
4. Publish a security advisory
5. Credit the reporter (if desired)

We ask that security researchers:

- Give us reasonable time to address vulnerabilities before public disclosure
- Make a good faith effort to avoid privacy violations, data destruction, and service interruption
- Do not access or modify data beyond what is necessary to demonstrate the vulnerability

## Security Update Process

Security updates will be released as follows:

1. **Critical vulnerabilities**: Patch released within 7 days
2. **High-severity vulnerabilities**: Patch released within 30 days
3. **Medium/Low-severity vulnerabilities**: Patch included in next regular release

Security patches will be clearly marked in release notes and may include:

- CVE identifiers (when applicable)
- Description of the vulnerability
- Affected versions
- Recommended actions for users

## Known Security Considerations

### Hardware Access Requirements

This tool requires direct hardware access and cannot be safely sandboxed in VMs or containers. This design constraint means:

- The tool must run with sufficient privileges to access PCIe devices
- It should only be deployed on secured, trusted systems
- Access control should be enforced at the operating system level

### Configuration Files

Configuration files (TOML) specify hardware modifications. Ensure:

- Configuration files are stored with appropriate permissions
- Configuration files are validated before use
- Configuration file paths are sanitized to prevent path traversal

### Input Validation

All user inputs (command-line arguments, configuration files) should be validated. If you discover inadequate input validation, please report it as a security issue.

## Contact

For security-related questions or concerns that are not vulnerability reports, please contact the project maintainers through the issue tracker with the label "security-question".

Thank you for helping keep TT-GLX-PARTITION and its users safe!
