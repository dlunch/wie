# Contributing to the project

## Project Layout

- `wie_backend`: provides system level services for apis.
- `wie_cli`: cli for local testing
- `wie_core_arm`: arm emulation
- `wie_jvm_support`: jvm support
- `wie_midp`, `wie_wipi_*`, `wie_skvm`: api implementation
- `wie_j2me`, `wie_skt`, `wie_ktf`, `wie_lgt`: platform specific logics
- `wie_util`, `test_utils`: shared helpers & test support

Web interface is on the private repository and occasionally built with wie's main branch.

## References

- WIPI Java API 1.1.1: https://nikita36078.github.io/J2ME_Docs/docs/WIPI_API_1_1_1
- WIPI 1.2.1 Spec (KO): http://strauss.cnu.ac.kr/research/wipi/download/WIPI%20V1.2.1_final(ST1.2.1).pdf
- Additional WIPI docs (KO) shipped with emulator: https://emulation.gametechwiki.com/index.php/Cellphone_emulators#Emulators_5
- SKVM API Archive: https://web.archive.org/web/20050503191803/http://developer.xce.co.kr:80/api/SKTAPI/allclasses-frame.html
- MIDP 2.0 (JSR-118) Overview: https://docs.oracle.com/javame/config/cldc/ref-impl/midp2.0/jsr118/overview-summary.html
