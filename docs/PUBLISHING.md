# Publishing Strategy

This document outlines the strategy for publishing rustbridge artifacts to public repositories.

## Platform Comparison

### crates.io (Rust)

**Accessibility**: Very low barrier to entry
- Sign up with GitHub account
- Run `cargo publish` from your workspace
- No vetting, no approval process for most crates
- Free for everyone

**Benefits even at low adoption**:
- Standard way Rust developers expect to consume libraries
- Automatic documentation hosting on docs.rs
- Semantic versioning enforcement
- Easy dependency management via Cargo.toml

**Recommendation**: Publish early. Even small Rust projects benefit from being on crates.io. It's so low-friction that distributing as source-only would be unusual and create more work for users.

### Maven Central (Java/Kotlin)

**Accessibility**: Moderate barrier, but individuals can do it
- Prove domain ownership OR use `io.github.<username>` namespace
- Create Sonatype JIRA account and request repository access
- Set up GPG key signing
- Configure Gradle publishing plugin
- Wait ~2 business days for initial approval
- After setup, publishing is `./gradlew publish`

**Benefits**:
- Standard distribution method for JVM libraries
- Corporate environments trust Maven Central artifacts
- Version immutability (can't change published artifacts)
- Automatic artifact mirroring and CDN distribution

**Drawbacks**:
- More bureaucracy than crates.io
- GPG signing requirement
- Stricter requirements (javadoc, sources JARs)

## Strategic Recommendation for rustbridge

### Rust Crates: Publish Now
- The multi-crate workspace structure is ready
- Publishing to crates.io signals maturity
- Makes it trivial for Rust developers to try it
- You can always yank versions if API changes drastically

### Java Artifacts: Consider Waiting If
- API is still in flux (breaking changes expected)
- You haven't decided on final package naming (`io.github.yourname` vs custom domain)
- You want more real-world testing first

**However**, publishing to Maven Central even pre-1.0 (using 0.x versions) is common and acceptable. Semantic versioning allows breaking changes in 0.x releases.

## Phased Rollout Plan

### Phase 1: Now
- Publish Rust crates to crates.io (low effort, high value)

### Phase 2: Now or Soon
- Set up Maven Central publishing infrastructure
- Use snapshots or local repositories for testing

### Phase 3: When Stable-ish
- Publish 0.1.0 to Maven Central
- Include clear "pre-1.0 API subject to change" messaging

### Phase 4: When Mature
- Cut 1.0.0 release with stability guarantees

## Key Insight

The "critical mass" argument applies more to marketing than infrastructure. The publishing infrastructure is worth setting up early because it forces you to think about:

- Versioning strategy
- API contracts
- Backwards compatibility

These considerations improve the project regardless of adoption levels.

## Who Can Publish?

Both platforms are accessible to individual developers, not just companies:

- **crates.io**: Anyone with a GitHub account
- **Maven Central**: Any individual willing to complete the setup process

There's no requirement for corporate backing or organizational affiliation.
