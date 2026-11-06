# Live verification

Byte-exact for each declared corpus and input domain under the recorded execution profile; finite file corpora are not universal equivalence.

Profile: `gnucobol-3.2.0-mingw32-vbisam-arnold-trembley`; reference architecture `i686-windows`.

| Case | Build | Scenarios | Result |
| --- | --- | ---: | --- |
| formatter | debug | 1 | byte_exact |
| fxsort | debug | 10 | byte_exact |
| sales | debug | 14 | byte_exact |
| formatter | release | 1 | byte_exact |
| fxsort | release | 10 | byte_exact |
| sales | release | 14 | byte_exact |

All reference-capture file hashes were unchanged after both verification builds.

The formatter corpus exhausts all 10,000 valid PIC 99V99 values twice. Other file corpora are finite examples, including declared malformed and missing-file cases. See case manifests for exact domains and any reference adaptation.
