# Web Gameplay Analytics

Browser gameplay uses virtual pageviews and built-in GA4 engagement in the existing property (`G-G6TBDLLPWK`). Native applications, including native canvas rendering, are excluded. Existing consent and pseudonymous browser identity remain authoritative; no user ID, fingerprint, backend, or additional storage.

## Pageviews
- Existing Google tag config sends the initial library pageview.
- Readiness sends `page_view` with the metadata title and same-origin `/games/<encoded title>` virtual URL.
- Actual library return restores its initial URL/title. Preparation failures add no duplicate library views; unloading does not claim a library return.
- Transitions use the previous virtual URL as `page_referrer` and update persistent tag context. Address bar and history do not change.

Only game titles are used, not filenames, filename-derived IDs, archive contents, or errors. Titles are author-provided; equal titles merge games. See Google's [pageview](https://developers.google.com/analytics/devguides/collection/ga4/views) and [SPA](https://developers.google.com/analytics/devguides/collection/ga4/single-page-applications) guidance.

## Reporting
No custom definitions are required. In Pages and screens, use page title or path, filtering paths to `/games/`, to inspect Views, Active users, and engagement metrics. Game views represent runs reaching readiness, not all launch attempts. Library returns allow another view for the next run of the same game.

Use built-in engagement metrics without filtering all data to `page_view`, which would exclude other automatic events. Engagement is GA page engagement, not exact emulator execution time: visible suspension or settings can still contribute. There is no custom duration timer, periodic gameplay report, or manual engagement/session identifier.

Use [User Explorer](https://support.google.com/analytics/answer/9283607?hl=en) for pseudonymous visitor timelines. Browser identity is not a verified person or cross-device identity; consent, cookie changes, retention, and reporting restrictions affect availability.

## Verification and Limits
Local verification uses actual-player browser integration and `npm run build:dev`, without Node tests or GA traffic. With property access, verify one pageview per transition, referrers, and subsequent automatic engagement context in DebugView using temporary debug mode. Deployed consent, ingestion, and engagement attribution require separate manual verification.

Collection is best effort. Blocking, consent, network failures, or abrupt termination can lose data. No retry queue or unload-blocking request is added; historical gameplay cannot be reconstructed.
