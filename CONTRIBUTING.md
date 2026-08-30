# Contributing to the project

## Project Layout

- `wie-backend`: provides system level services for apis.
- `wie-core-arm`: arm emulation
- `wie-jvm-support`: jvm support
- `wie-midp`, `wie-wipi-*`, `wie-skvm`: api implementation
- `wie-j2me`, `wie-skt`, `wie-ktf`, `wie-lgt`: platform specific logics
- `wie-util`, `test-utils`: shared helpers & test support

## References

- WIPI Java API 1.1.1: https://nikita36078.github.io/J2ME_Docs/docs/WIPI_API_1_1_1
- WIPI 1.2.1 Spec (KO): http://strauss.cnu.ac.kr/research/wipi/download/WIPI%20V1.2.1_final(ST1.2.1).pdf
- Additional WIPI docs (KO) shipped with emulator: https://emulation.gametechwiki.com/index.php/Cellphone_emulators#Wireless_Internet_Platform_for_Interoperability_.28WIPI.29
- SKVM API Archive: https://web.archive.org/web/20050503191803/http://developer.xce.co.kr:80/api/SKTAPI/allclasses-frame.html
- MIDP 2.0 (JSR-118) Overview: https://docs.oracle.com/javame/config/cldc/ref-impl/midp2.0/jsr118/overview-summary.html
