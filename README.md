# kommandozeilen_argumente

__Anmerkung__: Dies ist die englische ReadMe, für die deutsche Version siehe
[LIESMICH.md](https://github.com/spamviech/kommandozeilen_argumente/blob/main/LIESMICH.md).

TODO english version

Parser for command line arguments with optional automatic help generation.

Arguments are created with the provided associated functions,
the result type is specified with a type variable.
Arguments can be combined to more complex structures using more than one argument
with the `combine!` macro, or one of the dedicated `combineN` functions.

Arguments are identified by their long names and potentially their short names.
Long names are usually given after two minus characters `--long`.
Short names are usually given after one minus character `-short`.
Short names are expected to consist of only one
[Grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes).

All Strings can be adjusted, e.g. description of an argument in the help message.
Specialized functions for a german and english version are available if it is relevant.
Additionally, german an english synonyms are available.
Avoiding repetition of strings can be achieved using the `Language` type.

An Argument can have a default value, which is used if none of its names are used.
Without a default value the argument must be used, resulting in a parse error otherwise.

## Flags

Flags sind Argumente ohne Wert, sie können entweder aktiviert oder deaktiviert sein.
Meistens repräsentieren sie `bool`-Argumente, andere Typen werden aber ebenfalls unterstützt.

Angenommen Langnamen `flag`, Kurznamen `f` und invertiere_präfix `kein`,
eine Flag kann mit `--flag` oder `-f` aktiviert
und mit `--kein-flag` deaktiviert werden.

Existieren mehrere Flags mit Kurznamen `f`, `g` und `h`,
so können alle gleichzeitig mit `-fgh` aktiviert werden.

### Frühes Beenden

Eine besondere Art von Flags führt zu frühem beenden.
Sie können nicht deaktiviert werden und führen zu vorzeitigem Beenden unter Anzeigen einer Nachricht.
Typische Anwendungsfälle sind Anzeigen der aktuellen Version oder des Hilfe-Textes.

## Werte

Argumente können ebenfalls Werte spezifizieren.
Bei Langnamen wird der Wert getrennt von einem Leerzeichen, oder `=`-Zeichen angegeben.
Bei Kurznamen kann der Wert auch direkt im Anschluss an den Namen angegeben werden.
Die Konvertierung des Wertes wird aus `&OsStr` versucht.

Es ist möglich, alle erlaubten Werte im Hilfe-Text anzeigen zu lassen.

Angenommen Langnamen `--wert` und Kurznamen `-w` für ein Zahlen-Argument,
`--wert 3`, `--wert=3`, `-w 3`, `-w=3` und `-w3` werden alle mit Ergebnis `3` geparst.
Anstelle von `=` kann auch ein anderes Infix konfiguriert werden.

## Feature "derive"

Mit aktiviertem `derive`-Feature können die akzeptieren Kommandozeilen-Argumente
automatisch erzeugt werden.
Dazu wird das `Parse`-Trait für ein `struct` mit benannten Feldern implementiert.

Die Namen werden aus den Feld-Namen erzeugt,
der Langname ist der vollständige Feldname,
der Kurzname das erste Grapheme des Feldnamens.

Als Beschreibung im erzeugten Hilfe-Text wird der docstring des jeweiligen Feldes verwendet.

Zum parsen wird das `ParseArgument`-Trait verwendet.
Es ist implementiert für `bool`, `String`, Zahlentypen (`i8`, `u8`, `i16`, `u16`, ..., `f32`, `f64`),
`Option<T>` und Typen, die das `EnumArgument`-Trait implementieren.
Flag-Argumente werden für `bool`-Argumente erzeugt; diese sind standardmäßig deaktiviert.
Alle anderen Implementierungen erzeugen Wert-Argumente; `Option<T>` sind standardmäßig `None`,
alle anderen sind benötigte Argumente.
Das `EnumArgument`-Trait kann automatisch für ein `enum`, das keine Daten hält abgeleitet werden.
Für eine Verwendung als `ParseArgument` wird zusätzlich eine `Display`-Implementierung benötigt.

Das Standard-Verhalten kann über `#[kommandozeilen_argumente(<Optionen>)]`-Attribute beeinflusst werden.

Direkt am `struct` werden folgende Optionen unterstützt:

- `sprache: <sprache>` | `language: <language>`:
  Standard-Einstellung für einige Strings, Standard: `english`.
  Vorgefertigte Sprachen für `deutsch`, `englisch` und `english`.
- `version`: erzeuge eine `--version` Flag.
- `hilfe` | `help`: erzeuge eine Hilfe-Text.
- `hilfe(<opts>)`, `help(<opts>)`, `version(<opts>)`:
  Wie die Variante ohne opts, nur Kurzname ist standardmäßig deaktiviert. Mögliche Opts:
  - `lang_präfix: <präfix>` | `long_prefix: <prefix>`: Präfix vor Langnamen.
  - `lang: <name>`, `long [<namen>]`: Setze Langnamen explizit.
  - `kurz_präfix: <präfix>` | `short_prefix: <prefix>`: Präfix vor Kurznamen.
  - `kurz`: Setze Kurznamen als erstes Grapheme des originalen Langnamen.
  - `kurz: <name>`, `kurz: [<namen>]`: Setze Kurznamen explizit.
  - `sprache: <sprache>` | `language: <language>`: Sprache von Hilfe-Text und Standard-Namen.
  - `beschreibung: <programm-beschreibung>` | `description: <program_description>`:
    Nur bei `hilfe(<opts>)` und `help(<opts>)` verfügbar.
    Setze die im Hilfetext angezeigte Programm-Beschreibung.
- `case: sensitive`, `case: insensitive`:
  Alle Strings (Namen, Präfix, Infix) werden mit/ohne Berücksichtigung von
  Groß-/Kleinschreibung verglichen (Standard: `case: sensitive`).
- `case(<opts>)`:
  Genauere Einstellung zu Groß-/Kleinschreibung. Alle Opts haben die Form `<name>: <wert>`,
  mit `<wert>` entweder `sensitive` oder `insensitive`. Erlaubte Namen:
  - `lang_präfix` | `long_prefix`
  - `lang` | `long`
  - `kurz_präfix` | `short_prefix`
  - `kurz` | `short`
  - `invertiere_präfix` | `invert_prefix`
  - `invertiere_infix` | `invert_infix`
  - `wert_infix` | `value_infix`
- `lang_präfix: <präfix>` | `long_prefix: <prefix>`:
  Setze Standardwert für Präfix vor Langnamen, Standard `--`.
- `kurz_präfix: <präfix>` | `short_prefix: <prefix>`:
  Setze Standardwert für Präfix vor Kurznamen, Standard `-`.
- `invertiere_präfix: <string>` | `invert_prefix: <string>`:
  Setze Standardwert für Präfix zum invertieren einer Flag, Standard `kein` oder `no`.
- `invertiere_infix: <string>` | `invert_infix: <string>`:
  Setze Infix nach Präfix zum invertieren einer Flag, Standard `-`.
- `wert_infix: <string>` | `value_infix: <string>`:
  Setze Infix zum Angeben des Wertes im selben Argument, Standard `=`.
- `meta_var: <string>` | `meta_var: <string>`:
  Setze Standardwert für in der Hilfe angezeigte Meta-Variable, Standard `WERT` oder `VALUE`.

Vor Feldern werden folgende Optionen unterstützt:

- `glätten`/`flatten`: Verwende das Parse-Trait (übernehmen der konfigurierten Argumente).
- `FromStr`: Verwende das FromStr-Trait (benötigt Display für Wert und Fehler-Typ).
- `benötigt`/`required`: Entferne den konfigurierten Standard-Wert.
- `lang_präfix: <präfix>` | `long_prefix: <prefix>`: Präfix vor Langnamen.
- `lang: <name>` | `long: <name>`: Bestimme Langname explizit.
- `lang: [<namen>]` | `long: [<names>]`: Bestimme Langnamen explizit (Komma-getrennte Liste).
- `kurz_präfix: <präfix>` | `short_prefix: <prefix>`: Präfix vor Kurznamen.
- `kurz`/`short`: Verwende eine Kurzform, bestehend aus dem ersten Grapheme der Langform.
- `kurz: <wert>"`/`short: <value>"`: Verwende die spezifizierte Kurzform.
- `kurz: [<namen>]` | `short: [<names>]`: Bestimme Kurzformen explizit (Komma-getrennte Liste).
- `standard: <wert>` | `default: <value>`: Setzte den Standard-Wert.
- `invertiere_präfix: <string>` | `invert_prefix: <string>`: Setze Präfix zum invertieren einer Flag.
- `invertiere_infix: <string>` | `invert_infix: <string>`:
  Setze Infix nach Präfix zum invertieren einer Flag.
- `wert_infix: <string>` | `value_infix: <string>`:
  Setze Infix zum Angeben des Wertes im selben Argument.
- `meta_var: <string>`: Setzte die in der Hilfe angezeigt Meta-Variable.

## Beispiel

Ein einfaches Beispiel für ein `struct` mit 3 Flags und 2 Werten, erstellt über das
[Funktions-API](https://github.com/spamviech/kommandozeilen_argumente/blob/main/examples/funktion.rs),
bzw. [derive-API](https://github.com/spamviech/kommandozeilen_argumente/blob/main/examples/derive.rs)
können im [GitHub-Repository](https://github.com/spamviech/kommandozeilen_argumente/) eingesehen werden.
In beiden Fällen wird folgender Hilfe-Text erzeugt:

```cmd
kommandozeilen_argumente 0.2.0
Programm-Beschreibung.

funktion.exe [OPTIONEN]

OPTIONEN:
  --[kein]-flag                         Eine Flag mit Standard-Einstellungen [Standard: false]  
  --[kein]-(andere|namen) | -u          Eine Flag mit Standard-Einstellungen [Standard: false]  
  --[no]-benötigt         | -b          Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren.
  --wert(=| )WERT                       Ein String-Wert.
  --aufzählung(=| )VAR    | -a[=| ]VAR  Ein Aufzählung-Wert mit Standard-Wert und alternativer Meta-Variable. [Erlaubte Werte: Eins, Zwei, Drei | Standard: Zwei]
  --version               | -v          Zeige die aktuelle Version an.
  --hilfe                 | -h          Zeige diesen Text an.
```

## (Noch) Fehlende Features

- Unterargumente (subcommands)
- Positions-basierte Argumente
- Unterschiedlicher Standard-Wert für Name ohne Wert und Name kommt nicht vor
- Argumente unter Berücksichtigung von Groß- und Kleinschreibung
- Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)
