# GitHub Pages brief

This brief is intended for a future frontend agent such as Kimi K3 or Claude.

## Purpose

Build a polished public-facing display/download site for USTC Campus Agent. The site is not the application runtime and must not claim official USTC status.

## Content architecture

1. Hero: `USTC Campus Agent` and the one-sentence descriptor “面向科大学生的插件化校园智能体”.
2. Problem: campus information, course planning, opportunities, and workflows are fragmented.
3. Product: Plugins Market + Campus Opportunity Graph + bounded Agent.
4. Demo flow: install plugin → ask planning question → inspect evidence → change preference → disable plugin.
5. Architecture: Market, Rust authority core, source provenance, permissions.
6. Status: competition prototype; not official; source/data limitations.
7. Download: hidden or disabled until a verified GitHub Release exists.
8. Team/acknowledgement: no fake affiliations or logos.

## Design direction

- Direct and functional; no poetic “星图” branding unless later selected.
- Apple-like hierarchy, content deference, restrained motion.
- One signature visual: a small typed opportunity graph connecting requirement/course/source/preference nodes.
- Avoid generic purple gradients, fake dashboard screenshots, nested cards, or inflated metrics.

## Acceptance

- Lighthouse/accessibility basic pass;
- responsive mobile/tablet/desktop screenshots;
- no console errors;
- keyboard focus visible;
- no broken internal links;
- all download buttons either point to verified release assets or are marked “coming later”.

## Scaffold now vs later

Now: keep this brief and public-readiness checklist.

Later: add actual Pages framework, workflow, visual assets, release-download integration, browser QA, and publication gate.
