# Security Policy

K is an experimental compiler and should not yet be treated as production-grade security tooling.

The future KrumpyOS-native compiler and package build pipeline must assume that
package source is untrusted. Repository transport, signatures, checksums,
dependency locking, build isolation, installation permissions, and rollback
belong to `kpkg` and KrumpyOS. The compiler must not silently fetch source,
request root access, or install its own output.

## Reporting a vulnerability

Please do not publish details of a potentially exploitable compiler or build-system issue in a public issue first. Contact the repository maintainers privately when a maintainer contact is configured. Until then, record reproducible non-sensitive bugs through the issue tracker and avoid including secrets or private source code.

Reports should include the affected revision, host platform, reproduction case, and observed behavior. Do not test against systems or code you do not own.
