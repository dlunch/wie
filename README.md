# WIE

[Homepage](https://wie-site.dlunch.net) | [Try in browser](https://wie.dlunch.net)

<a href="https://play.google.com/store/apps/details?id=net.dlunch.wie">
  <img src="https://play.google.com/intl/en_us/badges/static/images/badges/en_badge_web_generic.png" height="48" alt="Get WIE on Google Play">
</a>

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

## Related projects

- [RustJava](https://github.com/dlunch/RustJava)
- [smaf](https://github.com/dlunch/smaf)
