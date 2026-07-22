# Product direction

## Decision

USTC Campus Agent is a campus-scoped Agent platform, not a general-purpose Agent framework. Its product spine is Plugins Market-first: user-visible capabilities are installed, authorized, disabled, upgraded, and audited through PluginPackage contracts.

## First flagship

- Plugin: `ustc.opportunity-graph`
- Product-facing name: Campus Opportunity Graph
- First vertical slice: Course Planning
- Optional display label for the slice: Course Compass

## Non-goals for the competition MVP

- no automatic enrollment/选课 clicking;
- no storage of raw USTC password or CAS session as product credential;
- no arbitrary third-party hosted code execution;
- no Android-native full experience before the Web/PWA loop is stable;
- no generic graph database or universal workflow engine introduced merely for architecture symmetry;
- no public repository or public download claims before public-readiness and release gates pass.

## Current naming

- Repository slug: `ustc-campus-agent`
- Product name: `USTC Campus Agent`
- Chinese name: TBD
- Chinese descriptor: 面向科大学生的插件化校园智能体

The name intentionally contains `Campus` to constrain the Agent to campus information, opportunity planning, workflows, and plugin-governed services.
