# kommandozeilen_argumente

Parser für Kommandozeilen-Argumente mit optionaler, potentiell deutscher, automatischer Hilfe-Generierung.

## derive-Attribute

trait ArgEnum ohne Attribute

trait Parse: alles über kommandozeilen_argumente-Attribut

Global an struct

- `deutsch` | `englisch` | `english`: Standard-Einstellung für einige Strings
- `version` | `version_deutsch` | `version_english`: erzeuge eine `--version` Flag
- `hilfe` | `help`: erzeuge eine Hilfe-Text

Vor Feldern

- hilfe aus docstring
- name wird langer name
- erstes grapheme wird kurzer name
- flags (bool-Werte) sind standardmäßig deaktiviert,
    ansonsten wird eine Implementierung über das ArgEnum-Trait verwendet
- `glätten`/`flatten`: verwende das Parse-Trait (übernehmen der konfigurierten Argumente)
- `benötigt`/`required`: entferne den konfigurierten Standard-Wert
- `standard: <wert>` | `default: <value>`: setzte den Standard-Wert
- `meta_var: <string>`: setzte die in der Hilfe angezeigt Meta-Variable
- `invertiere_prefix: <string>` | `invert_prefix: <string>`: setze Präfix zum invertieren einer Flag
