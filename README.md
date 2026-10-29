# WIE

[Homepage](https://wie-site.dlunch.net) | [Try in browser](https://wie.dlunch.net)

A standalone emulator for old mobile apps based on WIPI, SKVM or J2ME.

This project is dedicated to digital preservation and educational research. Our goal is to revive the legacy of classic mobile apps and allow them to be experienced in modern environments.

- [Contribution guide](https://github.com/dlunch/wie/blob/main/CONTRIBUTING.md)
- Architecture docs: [Emulator](docs/architecture.md) | [KTF](docs/ktf.md) | [LGT](docs/lgt.md)

## Frontend

The web and Android/iOS frontends are maintained in this repository under `wie-web` and `wie-app`.
Building the iOS app requires Xcode 27 or later and targets iOS 15 or later.

```bash
npm install
npm run build:dev   # development web build
npm run build:prod  # production web build
npm start           # web development server
```

Web CI deploys pull requests from this repository to the `wie-dev` Cloudflare Pages project,
using a stable `pr-<number>` branch alias. The deployment job summary and a bot comment link
to the preview; subsequent deployments update the content at the same URL when the PR changes
or reopens. Previews and their comment links remain available after the PR closes or merges.
Fork and Dependabot-triggered runs build without deploying. Preview deployments
reuse the release workflow's `CLOUDFLARE_API_TOKEN` secret and `CLOUDFLARE_ACCOUNT_ID` variable.

## Related projects

- [RustJava](https://github.com/dlunch/RustJava)
- [smaf](https://github.com/dlunch/smaf)
