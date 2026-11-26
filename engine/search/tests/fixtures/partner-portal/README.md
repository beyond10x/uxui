# partner-portal fixture

A copy of ESS's example `examples/partner-portal/` at tag `0.99.5` (commit
`3cdf39f157acced18513d506e84fffd49a72d58c`, github.com/beyond10x/ess, Apache-2.0): the document
and every file it needs to load. Unchanged; `ux-search` tests climb and measure its page
`tickets.detail`. The files were copied at an earlier tag and are the same bytes at this one: the
digests below hold for both.

| file | source | SHA-256 |
|---|---|---|
| `ui.yaml` | `examples/partner-portal/ui.yaml` | `bcb2db0cbdb5f213ee95e70e7da108af037da02a0965af2465ed1fba0c5cde4b` |
| `fixtures/forecast.yaml` | `examples/partner-portal/fixtures/forecast.yaml` | `38c4e1b158741dcabb48fce1ddfebe545cb87cbd88d24f5474c3c9d7a93bdee0` |
| `fixtures/index.yaml` | `examples/partner-portal/fixtures/index.yaml` | `75c21490e70d3e54f0a5afd999a549dbaa2d8a689f1f90f9a6b8b372a019eb21` |
| `fixtures/portal.yaml` | `examples/partner-portal/fixtures/portal.yaml` | `24167b66a5e7a3733c3768bdc9e56306afe3121db42ec392c2487c81e64c4f40` |
| `fixtures/scripts/activity.yaml` | `examples/partner-portal/fixtures/scripts/activity.yaml` | `04651293c033bc3cbdc4eaea0849ee674fb049b5aa92e973bc192253ca236b80` |
| `fixtures/scripts/metrics.yaml` | `examples/partner-portal/fixtures/scripts/metrics.yaml` | `5a0dbe111304d7ebc5f2874b08c305ec15b0b393ac70ede8eadbc5594c3d28ac` |
| `fixtures/scripts/ticket_chat.yaml` | `examples/partner-portal/fixtures/scripts/ticket_chat.yaml` | `9fff207abcc835d1928732c6ca0fa585d0cecd735de3688fc79e3b9f85ef9ab2` |
| `fixtures/scripts/tickets.yaml` | `examples/partner-portal/fixtures/scripts/tickets.yaml` | `9f6371ac9ea36b7dfc141b5661b73e820068598444505a865dbeeadc4490dcfe` |

Check from this directory: `sha256sum ui.yaml fixtures/*.yaml fixtures/scripts/*.yaml`.
