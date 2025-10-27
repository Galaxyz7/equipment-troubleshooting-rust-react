# Security Policy

## Supported Versions

We actively support and provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |
| < 2.0   | :x:                |

## Reporting a Vulnerability

We take the security of Equipment Troubleshooting System seriously. If you discover a security vulnerability, please follow these steps:

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report security vulnerabilities by emailing:
- **Email**: [security@example.com](mailto:security@example.com)
- **Subject**: "SECURITY: [Brief Description]"

### What to Include

Please include the following information in your report:
- **Description**: A clear description of the vulnerability
- **Impact**: Potential impact and attack scenario
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Proof of Concept**: If possible, include a PoC (without causing harm)
- **Suggested Fix**: If you have ideas on how to fix the vulnerability
- **Your Contact Information**: For follow-up questions

### Response Timeline

- **Initial Response**: Within 48 hours of report submission
- **Status Update**: Within 7 days with assessment and timeline
- **Resolution**: Critical vulnerabilities will be addressed within 30 days

### What to Expect

1. We will acknowledge receipt of your vulnerability report
2. We will investigate and confirm the vulnerability
3. We will develop and test a fix
4. We will release a security patch
5. We will publicly disclose the vulnerability (with credit to you, if desired)

## Security Best Practices

### For Administrators

**Authentication & Authorization**:
- Use strong, unique passwords (minimum 12 characters)
- Enable and review audit logs regularly via `/api/v1/admin/audit-logs`
- Limit admin access to trusted personnel only
- Use HTTPS in production environments

**Database Security**:
- Ensure `DATABASE_URL` is never committed to version control
- Use strong database passwords
- Restrict database access to necessary hosts only
- Enable SSL/TLS for database connections in production

**Environment Variables**:
- Keep `.env` files secure and never commit them
- Use environment-specific `.env` files (`.env.production`, `.env.staging`)
- Rotate `JWT_SECRET` regularly (minimum 32 characters)
- Use separate secrets for different environments

**Rate Limiting**:
- Monitor rate limit metrics for abuse patterns
- Adjust rate limits based on your traffic patterns
- Review blocked requests periodically

**Session Management**:
- Regularly clean up old sessions using the admin dashboard
- Monitor for suspicious session activity
- Set appropriate session timeouts

### For Developers

**Code Security**:
- Never commit secrets, API keys, or credentials
- Use parameterized queries (SQLx) to prevent SQL injection
- Validate and sanitize all user inputs
- Follow principle of least privilege

**Dependencies**:
- Regularly update dependencies: `cargo update` and `npm update`
- Review security advisories: `cargo audit`
- Monitor for npm vulnerabilities: `npm audit`

**API Security**:
- All admin routes require authentication (`/api/v1/admin/*`)
- JWT tokens expire after 24 hours
- CORS is configured to restrict origins
- Rate limiting prevents abuse

## Security Features

### Built-in Protections

**Authentication & Authorization**:
- JWT-based authentication with HS256 algorithm
- Role-based access control (Admin/User roles)
- Token expiration and refresh mechanism
- Password hashing with bcrypt (cost factor 12)

**Audit Logging**:
- Comprehensive audit trail for all admin actions
- IP address tracking for security monitoring
- Immutable audit logs in PostgreSQL
- Queryable audit history via admin API

**SQL Injection Prevention**:
- Parameterized queries using SQLx
- Compile-time query validation
- No dynamic SQL construction with user input

**Rate Limiting**:
- Per-IP rate limiting on all endpoints
- Configurable limits per route
- Automatic request throttling
- DDoS protection layer

**Security Headers**:
- `X-Frame-Options: DENY` (clickjacking protection)
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security` (HSTS)

**Input Validation**:
- Server-side validation for all inputs
- Type-safe validation with TypeScript/Rust
- Length limits on text fields
- Semantic ID format validation

### Data Protection

**At Rest**:
- PostgreSQL database with user-defined encryption options
- Audit logs retained indefinitely for compliance
- Secure credential storage

**In Transit**:
- HTTPS recommended for production
- Encrypted JWT tokens
- Secure WebSocket connections (if implemented)

## Known Security Considerations

### Session Data Privacy

User session data contains troubleshooting history which may include:
- Equipment details
- Site information
- Technician identifiers

**Recommendations**:
- Review your data retention policy
- Implement periodic session cleanup
- Consider GDPR/privacy law compliance for your jurisdiction

### Admin Access

The admin panel provides significant control:
- Full CRUD operations on all data
- Session management and deletion
- Audit log access

**Recommendations**:
- Limit admin accounts to essential personnel
- Review admin activity via audit logs
- Use strong authentication for admin accounts
- Consider implementing 2FA for admin access (future enhancement)

## Compliance

### Audit Logging

All administrative actions are logged with:
- User ID and email
- Action type and timestamp
- Resource affected
- IP address
- Detailed context (JSON)

Access audit logs via:
```bash
GET /api/v1/admin/audit-logs
```

### Data Retention

**Audit Logs**: Retained indefinitely
**Sessions**: Configurable retention, manual cleanup via admin panel
**User Data**: Retained while account is active

## Security Roadmap

Future security enhancements planned:
- [ ] Two-factor authentication (2FA) for admin accounts
- [ ] Automated security scanning in CI/CD
- [ ] Enhanced password policies (complexity requirements)
- [ ] Session timeout configuration
- [ ] IP whitelisting for admin panel
- [ ] Automated audit log analysis and alerting

## Contact

For security concerns or questions:
- **Email**: security@example.com
- **Response Time**: Within 48 hours

## Acknowledgments

We appreciate the security research community's efforts in responsibly disclosing vulnerabilities. Security researchers who report valid vulnerabilities will be acknowledged in our security advisories (with permission).

---

**Last Updated**: 2025-01-26
**Version**: 2.0.0
